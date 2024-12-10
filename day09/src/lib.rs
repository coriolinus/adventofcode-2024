use std::path::Path;

use dlv_list::{Index, VecList};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumIs)]
enum Block {
    File(u32),
    Free,
}

#[derive(Debug)]
struct FilesystemEntry {
    item: Block,
    size: u16,
}

impl FilesystemEntry {
    fn new(item: Block, size: u16) -> Self {
        Self { item, size }
    }

    /// Split the front `size` of self into a new item with zeroized pointers.
    ///
    /// The current item has its size reduced by `size` but remains in place.
    fn split_at(mut self, size: u16) -> (Self, Self) {
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

        fs.push_back(FilesystemEntry::new(block, size.into()));
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

/// Remove a file from the filesystem, returning both it and the index of the free space replacing it.
///
/// This function also scans for adjacent free space in the filesystem and consolidates it,
/// preserving the pattern that no two files have more than one free space between them,
/// and also preserving the total number of blocks in the filesystem.
fn remove_file(
    fs: &mut Filesystem,
    cursor: Index<FilesystemEntry>,
) -> (FilesystemEntry, Index<FilesystemEntry>) {
    let out = FilesystemEntry::new(fs[cursor].item, fs[cursor].size);
    fs[cursor].item = Block::Free;

    let mut leftmost_free = cursor;
    while let Some(peek) = fs.get_previous_index(leftmost_free) {
        if fs[peek].item.is_free() {
            leftmost_free = peek;
        } else {
            break;
        }
    }

    let mut rightmost_free = cursor;
    while let Some(peek) = fs.get_next_index(rightmost_free) {
        if fs[peek].item.is_free() {
            rightmost_free = peek;
        } else {
            break;
        }
    }

    let left_of_leftmost_free = fs.get_previous_index(leftmost_free);
    let mut total_free_space = 0;
    let mut cursor = leftmost_free;
    loop {
        let next = (cursor != rightmost_free).then(|| {
            fs.get_next_index(cursor)
                .expect("cursor was left of rightmost free")
        });

        let entry = fs.remove(cursor).expect("cursor must still be valid");
        debug_assert!(entry.item.is_free());
        total_free_space += entry.size;

        match next {
            Some(next) => cursor = next,
            None => break,
        }
    }

    let consolidated_free_space = FilesystemEntry::new(Block::Free, total_free_space);
    let idx = match left_of_leftmost_free {
        Some(idx) => fs.insert_after(idx, consolidated_free_space),
        None => fs.push_front(consolidated_free_space),
    };

    (out, idx)
}

fn compact_filesystem_no_fragments(fs: &mut Filesystem) -> Result<(), Error> {
    let mut lowest_checked_file_id = !0;
    let mut cursor = fs.back_index().ok_or(Error::NoSolution)?;

    loop {
        macro_rules! cursor_continue {
            () => {
                match fs.get_previous_index(cursor) {
                    Some(prev) => cursor = prev,
                    None => break,
                }
                continue;
            };
        }

        let entry = &fs[cursor];
        match entry.item {
            Block::Free => {
                cursor_continue!();
            }
            Block::File(id) if id >= lowest_checked_file_id => {
                cursor_continue!();
            }
            Block::File(id) => lowest_checked_file_id = id,
        }

        #[cfg(test)]
        eprintln!("{}; ({:?}, {})", fs_to_str(fs), entry.item, entry.size);

        // cursor is now pointing at the highest-id file we have not yet examined
        // we have to scan from the start to find a block where it might fit\

        // we can't actually keep the free idx here, as removing a file can then invalidate it.
        // but we want to check here, before we actually remove the file. So let's duplicate some work!
        let mut encountered_self = false;
        if !fs.indices().any(|idx| {
            encountered_self |= idx == cursor;
            !encountered_self && fs[idx].item.is_free() && fs[idx].size >= entry.size
        }) {
            cursor_continue!();
        };

        // found a free spot that's big enough
        // get the next cursor here, in case removing the entry messes with it
        let (entry, free_cursor) = remove_file(fs, cursor);

        // re-find the free space to get a known-good index
        encountered_self = false;
        let free_idx = fs
            .indices()
            .find(|&idx| {
                encountered_self |= idx == cursor;
                !encountered_self && fs[idx].item.is_free() && fs[idx].size >= entry.size
            })
            .expect("we found enough free space before, we should find it again");

        match entry.size.cmp(&fs[free_idx].size) {
            std::cmp::Ordering::Less => {
                fs[free_idx].size -= entry.size;
                fs.insert_before(free_idx, entry);
            }
            std::cmp::Ordering::Equal => {
                fs[free_idx] = entry;
            }
            std::cmp::Ordering::Greater => {
                unreachable!("we just checked that free size >= entry size")
            }
        }

        match fs.get_previous_index(free_cursor) {
            Some(next) => cursor = next,
            None => break,
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
    let data = std::fs::read_to_string(input)?;
    let mut fs = fs_from_str(data.trim())?;

    compact_filesystem_no_fragments(&mut fs)?;
    let checksum = checksum(&fs);
    println!("checksum, no fragments: {checksum}");

    Ok(())
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

    #[test]
    fn compact_long_no_fragments_example() {
        let mut fs = fs_from_str("2333133121414131402").unwrap();
        compact_filesystem_no_fragments(&mut fs).unwrap();
        assert_eq!(fs_to_str(&fs), "00992111777.44.333....5555.6666.....8888");
        assert_eq!(checksum(&fs), 2858);
    }
}
