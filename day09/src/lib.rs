use std::path::Path;

use dlv_list::VecList;

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumIs)]
enum Block {
    File(u32),
    Free,
}

struct FilesystemEntry {
    item: Block,
    size: u8,
}

impl FilesystemEntry {
    fn new(item: Block, size: u8) -> Self {
        Self { item, size }
    }

    /// Split the front `size` of self into a new item with zeroized pointers.
    ///
    /// The current item has its size reduced by `size` but remains in place.
    fn split_at(mut self, size: u8) -> (Self, Self) {
        debug_assert!(self.size > size);
        let front = Self::new(self.item, size);
        self.size -= size;
        (front, self)
    }
}

type Filesystem = VecList<FilesystemEntry>;

fn fs_from_str(s: &str) -> Result<Filesystem, Error> {
    let mut is_file = true;
    let mut file_id = 0;

    let mut fs = Filesystem::with_capacity(s.len());

    for b in s.as_bytes().iter().copied() {
        if !b.is_ascii_digit() {
            return Err(Error::NotANumber);
        }

        let size = b - b'0';
        let block;
        if is_file {
            block = Block::File(file_id);
            file_id += 1;
        } else {
            block = Block::Free;
        }
        is_file = !is_file;

        fs.push_back(FilesystemEntry::new(block, size));
    }

    Ok(fs)
}

#[cfg(test)]
fn fs_to_str(fs: &Filesystem) -> String {
    use std::fmt::Write as _;

    let mut out = String::with_capacity(fs.iter().map(|entry| entry.size as usize).sum());
    for entry in fs.iter() {
        for _ in 0..entry.size {
            match entry.item {
                Block::File(id) => {
                    if id < 10 {
                        write!(&mut out, "{id}").expect("writing to a string always succeeds");
                    } else {
                        write!(&mut out, "({id})").expect("writing to a string always succeeds");
                    }
                }
                Block::Free => out.push('.'),
            }
        }
    }
    out
}

fn compact_filesystem(fs: &mut Filesystem) -> Result<(), Error> {
    let mut cursor = fs.front_index().ok_or(Error::NoSolution)?;

    loop {
        // termination conditions
        let Some(free) = fs.get(cursor) else {
            // if the cursor has been invalidated, then we must have just popped this item
            // off the back, which therefore means that we're done now
            break;
        };
        #[cfg(test)]
        eprintln!("{}", fs_to_str(fs));
        if free.item.is_file() {
            if let Some(next) = fs.get_next_index(cursor) {
                cursor = next;
                continue;
            } else {
                // we can't advance the cursor any more, so we must be done
                break;
            }
        }

        // if we're here, the cursor is pointing to a free space, so let's get the last file
        let Some(file) = fs.pop_back() else {
            break;
        };
        if file.item.is_free() {
            continue;
        }

        // at this point the cursor is pointing at a free space and we've popped the trailing file,
        // so let's handle that condition
        let advance = file.size >= fs[cursor].size;
        match file.size.cmp(&fs[cursor].size) {
            std::cmp::Ordering::Less => {
                fs[cursor].size -= file.size;
                fs.insert_before(cursor, file);
            }
            std::cmp::Ordering::Equal => fs[cursor] = file,
            std::cmp::Ordering::Greater => {
                let (first_part, second_part) = file.split_at(fs[cursor].size);
                fs[cursor] = first_part;
                fs.push_back(second_part);
            }
        }

        if advance {
            match fs.get_next_index(cursor) {
                Some(next) => cursor = next,
                None => break,
            }
        }
    }

    // cleanup trailing free space
    while let Some(back) = fs.back() {
        if back.item.is_free() {
            fs.pop_back();
        } else {
            break;
        }
    }

    debug_assert!(fs.iter().all(|entry| entry.item.is_file()));
    Ok(())
}

fn checksum(fs: &Filesystem) -> u64 {
    let mut sum = 0;
    let mut position = 0;
    for entry in fs.iter() {
        if let Block::File(id) = entry.item {
            for p in position..(position + entry.size as u64) {
                sum += p * id as u64;
            }
        }
        position += entry.size as u64;
    }
    sum
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let data = std::fs::read_to_string(input)?;
    let mut fs = fs_from_str(data.trim())?;

    compact_filesystem(&mut fs)?;
    let checksum = checksum(&fs);
    println!("checksum: {checksum}");

    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    unimplemented!("input file: {:?}", input)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("non-numeric digit found")]
    NotANumber,
    #[error("no solution found")]
    NoSolution,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_short_example() {
        let fs = fs_from_str("12345").unwrap();
        assert_eq!(fs_to_str(&fs), "0..111....22222");
    }

    #[test]
    fn parse_long_example() {
        let fs = fs_from_str("2333133121414131402").unwrap();
        assert_eq!(fs_to_str(&fs), "00...111...2...333.44.5555.6666.777.888899");
    }

    #[test]
    fn compact_short_example() {
        let mut fs = fs_from_str("12345").unwrap();
        compact_filesystem(&mut fs).unwrap();
        assert_eq!(fs_to_str(&fs), "022111222");
    }

    #[test]
    fn compact_long_example() {
        let mut fs = fs_from_str("2333133121414131402").unwrap();
        compact_filesystem(&mut fs).unwrap();
        assert_eq!(fs_to_str(&fs), "0099811188827773336446555566");
        assert_eq!(checksum(&fs), 1928);
    }
}
