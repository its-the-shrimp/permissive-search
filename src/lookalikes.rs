use sort_const::const_quicksort;

/// Returns all characters that `ch` could've been a misclick of.
///
/// E.g. if the user typed in `a`, it could mean that they meant `a`, or (assuming their keybaord
/// is in the QWERTY layout) they've misclicked one of the following: `q`, `w`, `s`, `x`, `z`
pub fn qwerty_misclicks(ch: char) -> impl Iterator<Item = char> + Clone {
    const fn shifted(ch: char) -> char {
        if ch.is_ascii_alphanumeric() {
            ch.to_ascii_uppercase()
        } else {
            match ch {
                ';' => ':',
                ',' => '<',
                '.' => '>',
                '/' => '?',
                _ => ch,
            }
        }
    }

    const LAYOUT: [[char; 10]; 3] = [
        ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'],
        ['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';'],
        ['z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/'],
    ];

    static MISCLICKS: [(char, [char; 8]); 30] = {
        const N_ROWS: usize = LAYOUT.len();
        const N_COLS: usize = LAYOUT[0].len();

        let mut res = [('\0', ['\0'; 8]); 30];
        let mut row = 0;
        while row < N_ROWS {
            let mut col = 0;
            while col < N_COLS {
                let (ch, slot) = &mut res[row * N_COLS + col];
                *ch = LAYOUT[row][col];
                if row > 0 {
                    if col > 0 {
                        slot[0] = LAYOUT[row - 1][col - 1];
                    }
                    slot[1] = LAYOUT[row - 1][col];
                    if col < N_COLS - 1 {
                        slot[2] = LAYOUT[row - 1][col + 1];
                    }
                }
                if col > 0 {
                    slot[3] = LAYOUT[row][col - 1];
                }
                if col < N_COLS - 1 {
                    slot[4] = LAYOUT[row][col + 1];
                }
                if row < N_ROWS - 1 {
                    if col > 0 {
                        slot[5] = LAYOUT[row + 1][col - 1];
                    }
                    slot[6] = LAYOUT[row + 1][col];
                    if col < N_COLS - 1 {
                        slot[7] = LAYOUT[row + 1][col + 1];
                    }
                }
                col += 1;
            }
            row += 1;
        }

        const_quicksort!(res, |lhs, rhs| lhs.0 < rhs.0)
    };

    MISCLICKS
        .binary_search_by_key(&ch.to_ascii_lowercase(), |(ch, _)| *ch)
        .map_or(&[][..], |i| &MISCLICKS[i].1[..])
        .iter()
        .filter(|c| **c != '\0')
        .flat_map(move |&c| [c, shifted(c)])
}
