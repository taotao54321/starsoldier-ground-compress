use std::collections::{hash_map::Entry, HashMap};
use std::convert::{TryFrom, TryInto};

use crate::ROW_WIDTH;

pub fn encode(buf: impl AsRef<[u8]>, origin: usize) -> eyre::Result<Vec<u8>> {
    let buf = buf.as_ref();

    eyre::ensure!(
        buf.len() % ROW_WIDTH == 0,
        "input size is not multiple of {}",
        ROW_WIDTH
    );

    let mut encoded = Vec::with_capacity(buf.len());

    let mut row_addr = HashMap::<[u8; ROW_WIDTH], usize>::new();
    for row in buf.chunks(ROW_WIDTH) {
        let row: [u8; ROW_WIDTH] = row.try_into().unwrap();

        // RLE により最小 2Byte まで縮む可能性がある。
        // アドレス参照は 3Byte なので、先に RLE を試す必要がある。
        // RLE で 3Byte になる場合、NES 側の計算量的に RLE の方が有利。
        let rle = encode_row(&row);
        if rle.len() <= 3 {
            encoded.extend_from_slice(&rle);
            continue;
        }

        match row_addr.entry(row) {
            Entry::Occupied(o) => {
                let addr = u16::try_from(*o.get()).expect("address overflow");
                encoded.push(0xDB);
                encoded.push(u8::try_from(addr & 0xFF).unwrap());
                encoded.push(u8::try_from(addr >> 8).unwrap());
            }
            Entry::Vacant(v) => {
                let addr = origin + encoded.len();
                v.insert(addr);
                encoded.extend_from_slice(&rle);
            }
        }
    }

    Ok(encoded)
}

macro_rules! chmin {
    ($xmin:expr, $x:expr) => {{
        if $x < $xmin {
            $xmin = $x;
            true
        } else {
            false
        }
    }};
}

/// `row` の最適な符号を返す。
///
/// 動的計画法を用いる。
fn encode_row(row: &[u8; ROW_WIDTH]) -> Vec<u8> {
    const INF: usize = usize::MAX;

    // dp[i]: 入力 i バイトをエンコードしたときの出力サイズの最小値
    // from[i]: dp[i] を実現したときに頂点 i へ入る辺 (端点, ランレングス単位)
    let mut dp = [INF; ROW_WIDTH + 1];
    let mut from = [(usize::MAX, usize::MAX); ROW_WIDTH + 1];
    dp[0] = 0;

    // 配るDP
    for i in 0..ROW_WIDTH {
        // RLE なし
        if chmin!(dp[i + 1], dp[i] + 1) {
            from[i + 1] = (i, 0);
        }

        // RLE あり
        let j_max = 4.min(ROW_WIDTH - i);
        for j in 1..=j_max {
            let seq = &row[i..][..j];

            let r_min = i + 2 * j;
            for r in (r_min..=ROW_WIDTH).step_by(j) {
                if &row[r - j..r] != seq {
                    break;
                }
                if j == 1 && r == i + 2 {
                    continue;
                }
                if chmin!(dp[r], dp[i] + 1 + j) {
                    from[r] = (i, j);
                }
            }
        }
    }

    // 経路復元
    let mut trace = vec![];
    {
        let mut i = ROW_WIDTH;
        while i != 0 {
            let (i_pre, unit) = from[i];
            let rep = if unit == 0 {
                0
            } else {
                debug_assert_eq!((i - i_pre) % unit, 0);
                (i - i_pre) / unit
            };
            trace.push((i_pre, unit, rep));
            i = i_pre;
        }
    }
    trace.reverse();

    let mut res = vec![];

    // 符号化
    for (i, unit, rep) in trace {
        if unit == 0 {
            res.push(row[i]);
            continue;
        }

        let seq = &row[i..][..unit];

        match unit {
            1 => {
                debug_assert!((3..=20).contains(&rep));
                res.push(0xEB + rep as u8);
            }
            2 => {
                debug_assert!((2..=10).contains(&rep));
                res.push(0xE3 + rep as u8);
            }
            3 => {
                debug_assert!((2..=6).contains(&rep));
                res.push(0xDE + rep as u8);
            }
            4 => {
                debug_assert!((2..=5).contains(&rep));
                res.push(0xDA + rep as u8);
            }
            _ => unreachable!(),
        }
        res.extend_from_slice(seq);
    }

    res
}
