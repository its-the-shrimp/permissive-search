/// Returns `(row, col)`
const fn find_char<const N_ROWS: usize, const N_COLS: usize>(
    ch: char,
    layout: &[[char; N_COLS]; N_ROWS],
) -> Option<(usize, usize)> {
    let mut row = 0;
    while row < N_ROWS {
        let mut col = 0;
        while col < N_COLS {
            if layout[row][col] == ch {
                return Some((row, col));
            }
            col += 1;
        }
        row += 1;
    }

    None
}

/// Returns all characters that `ch` could've been a misclick of.
///
/// E.g. if the user typed in `a`, it could mean that they meant `a`, or (assuming their keybaord
/// is in the QWERTY layout) they've misclicked one of the following: `q`, `w`, `s`, `x`, `z`
pub fn qwerty_misclicks(ch: char) -> impl Iterator<Item = char> + Clone {
    static LAYOUT: [[char; 10]; 4] = [
        ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'],
        ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'],
        ['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';'],
        ['z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/'],
    ];
    static SHIFTED_LAYOUT: [[char; 10]; 4] = [
        ['!', '@', '#', '$', '%', '^', '&', '*', '(', ')'],
        ['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P'],
        ['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', ':'],
        ['Z', 'X', 'C', 'V', 'B', 'N', 'M', '<', '>', '?'],
    ];

    /// The number of ASCII characters that are printable & typeable
    const N_TYPEABLES: usize = (b'~' - b' ') as usize;

    /// The maximum number of misclicks this iterator can produce
    const N_MISCLICKS: usize = 17;

    static MISCLICKS: [[char; N_MISCLICKS]; N_TYPEABLES] = {
        const N_ROWS: usize = LAYOUT.len();
        const N_COLS: usize = LAYOUT[0].len();
        assert!(size_of_val(&LAYOUT) == size_of_val(&SHIFTED_LAYOUT));

        let mut res = [['\0'; 17]; N_TYPEABLES];
        let mut next_byte = b' ';
        while next_byte <= b'~' {
            let ch = next_byte as char;

            // Assembling the set
            let byte = next_byte;
            next_byte += 1;
            let (row, col, toggled) = if let Some((row, col)) = find_char(ch, &LAYOUT) {
                (row, col, SHIFTED_LAYOUT[row][col])
            } else if let Some((row, col)) = find_char(ch, &SHIFTED_LAYOUT) {
                (row, col, LAYOUT[row][col])
            } else {
                continue;
            };
            let (toggled_p, set) = res[(byte - b' ') as usize].split_first_mut().unwrap();
            *toggled_p = toggled;

            if row > 0 {
                if col > 0 {
                    set[0] = LAYOUT[row - 1][col - 1];
                    set[8] = SHIFTED_LAYOUT[row - 1][col - 1];
                }
                set[1] = LAYOUT[row - 1][col];
                set[9] = SHIFTED_LAYOUT[row - 1][col];
                if col < N_COLS - 1 {
                    set[2] = LAYOUT[row - 1][col + 1];
                    set[10] = SHIFTED_LAYOUT[row - 1][col + 1];
                }
            }
            if col > 0 {
                set[3] = LAYOUT[row][col - 1];
                set[11] = SHIFTED_LAYOUT[row][col - 1];
            }
            if col < N_COLS - 1 {
                set[4] = LAYOUT[row][col + 1];
                set[12] = SHIFTED_LAYOUT[row][col + 1];
            }
            if row < N_ROWS - 1 {
                if col > 0 {
                    set[5] = LAYOUT[row + 1][col - 1];
                    set[13] = SHIFTED_LAYOUT[row + 1][col - 1];
                }
                set[6] = LAYOUT[row + 1][col];
                set[14] = SHIFTED_LAYOUT[row + 1][col];
                if col < N_COLS - 1 {
                    set[7] = LAYOUT[row + 1][col + 1];
                    set[15] = SHIFTED_LAYOUT[row + 1][col + 1];
                }
            }

            // Moving all NULs to the end to enable short-curcuiting on the first NUL
            let mut shift = 0;
            let mut i = 0;
            while i < 16 {
                if set[i] == '\0' {
                    shift += 1;
                } else {
                    set.swap(i, i - shift);
                }
                i += 1;
            }
        }

        res
    };

    u32::from(ch)
        .checked_sub(b' '.into())
        .and_then(|i| MISCLICKS.get(i as usize))
        .unwrap_or(&['\0'; N_MISCLICKS])
        .iter()
        .copied()
        .filter(|c| *c != '\0')
}
