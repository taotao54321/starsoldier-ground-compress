use crate::ROW_WIDTH;

pub fn decode(buf: impl AsRef<[u8]>, origin: usize) -> eyre::Result<Vec<u8>> {
    let buf = buf.as_ref();

    let mut decoded = Vec::new();

    let mut offset = 0;
    while offset < buf.len() {
        decode_row(buf, origin, &mut offset, &mut decoded)?;
    }

    Ok(decoded)
}

fn decode_row(
    buf: &[u8],
    origin: usize,
    offset: &mut usize,
    decoded: &mut Vec<u8>,
) -> eyre::Result<()> {
    if peek(buf, offset)? == 0xDB {
        *offset += 1;
        let mut offset_ref = read_addr(buf, offset)? - origin;
        decode_row_rle(buf, &mut offset_ref, decoded)?;
    } else {
        decode_row_rle(buf, offset, decoded)?;
    }

    Ok(())
}

fn decode_row_rle(buf: &[u8], offset: &mut usize, decoded: &mut Vec<u8>) -> eyre::Result<()> {
    let mut cell_count = 0;
    while cell_count < ROW_WIDTH {
        let first = read(buf, offset)?;
        if first < 0xDC {
            decoded.push(first);
            cell_count += 1;
            continue;
        }

        let (rep, seq) = match first {
            0xEE..=0xFF => (first - 0xEB, read_slice(buf, offset, 1)?),
            0xE5..=0xED => (first - 0xE3, read_slice(buf, offset, 2)?),
            0xE0..=0xE4 => (first - 0xDE, read_slice(buf, offset, 3)?),
            0xDC..=0xDF => (first - 0xDA, read_slice(buf, offset, 4)?),
            _ => unreachable!(),
        };
        for _ in 0..rep {
            decoded.extend_from_slice(seq);
            cell_count += seq.len();
        }
    }
    eyre::ensure!(cell_count == ROW_WIDTH, "row overflow");

    Ok(())
}

fn read_slice<'buf, 'offset>(
    buf: &'buf [u8],
    offset: &'offset mut usize,
    len: usize,
) -> eyre::Result<&'buf [u8]> {
    let res = buf
        .get(*offset..*offset + len)
        .ok_or_else(|| eyre::eyre!("slice range out of bounds"));
    *offset += len;
    res
}

fn read_addr(buf: &[u8], offset: &mut usize) -> eyre::Result<usize> {
    let lo = read(buf, offset)?;
    let hi = read(buf, offset)?;
    Ok(usize::from(lo) | (usize::from(hi) << 8))
}

fn read(buf: &[u8], offset: &mut usize) -> eyre::Result<u8> {
    let res = peek(buf, offset)?;
    *offset += 1;
    Ok(res)
}

fn peek(buf: &[u8], offset: &usize) -> eyre::Result<u8> {
    buf.get(*offset)
        .copied()
        .ok_or_else(|| eyre::eyre!("offset out of bounds"))
}
