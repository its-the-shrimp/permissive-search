use {
    crossterm::{
        ExecutableCommand,
        cursor::{MoveToColumn, MoveToNextLine, MoveToPreviousLine},
        event::{self, Event, KeyCode, KeyModifiers},
        style::Print,
        terminal::{
            Clear,
            ClearType::{CurrentLine, FromCursorDown, UntilNewLine},
            ScrollUp, disable_raw_mode, enable_raw_mode,
        },
    },
    sort_const::const_quicksort,
    std::{
        env::args_os,
        error::Error,
        fs::File,
        io::{self, BufRead, BufReader, Write, stdout},
        mem::swap,
        process::ExitCode,
    },
};

const PROMPT: &str = "> ";
const N_LINES: u16 = 10;

fn qwerty_misclicks(ch: char) -> impl Iterator<Item = char> + Clone {
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

    let is_shifted = ch.is_ascii_uppercase();
    MISCLICKS
        .binary_search_by_key(&ch.to_ascii_lowercase(), |(ch, _)| *ch)
        .map_or(&[][..], |i| &MISCLICKS[i].1[..])
        .iter()
        .filter(|c| **c != '\0')
        .map(move |&c| if is_shifted { shifted(c) } else { c })
}

#[derive(Debug, Default)]
struct SearchTree {
    nodes: Vec<(char, SearchTree)>,
    end: Option<usize>,
}

impl SearchTree {
    fn get(&self, index: char) -> Option<&Self> {
        self.nodes
            .binary_search_by_key(&index, |(ch, _)| *ch)
            .ok()
            .map(|i| &self.nodes[i].1)
    }

    fn push(&mut self, key: &str, index: usize) {
        let mut iter = key.chars();
        let Some(ch) = iter.next() else {
            self.end = Some(index);
            return;
        };

        let i = match self.nodes.binary_search_by_key(&ch, |(ch, _)| *ch) {
            Ok(i) => i,
            Err(i) => {
                self.nodes.insert(i, (ch, SearchTree::default()));
                i
            }
        };

        self.nodes[i].1.push(iter.as_str(), index)
    }

    fn print(&self, dst: &mut impl Write, indent: usize) -> io::Result<()> {
        for (ch, node) in &self.nodes {
            for _ in 0..indent {
                write!(dst, "│ ")?;
            }
            writeln!(dst, "{ch}")?;
            node.print(dst, indent + 1)?;
        }

        Ok(())
    }

    fn for_each_base<E>(&self, f: &mut impl FnMut(usize) -> Result<(), E>) -> Result<(), E> {
        if let Some(end) = self.end {
            f(end)?;
        }
        for node in &self.nodes {
            node.1.for_each_base(f)?;
        }
        Ok(())
    }

    fn for_each<E>(&self, mut f: impl FnMut(usize) -> Result<(), E>) -> Result<(), E> {
        self.for_each_base(&mut f)
    }
}

fn inner_main() -> Result<(), Box<dyn Error>> {
    let filename = args_os()
        .nth(1)
        .ok_or("Please provide the name of the file to read lines from")?;

    let file = File::open(&filename).map_err(|e| format!("Failed to open {filename:?}: {e}"))?;
    let lines = BufReader::new(file)
        .lines()
        .collect::<io::Result<Vec<_>>>()
        .map_err(|e| format!("Failed to read the contents of {filename:?}: {e}"))?;

    let mut root = SearchTree::default();
    for (index, line) in lines.iter().enumerate() {
        root.push(line, index);
    }

    enable_raw_mode()?;
    let mut stdout = stdout().lock();
    let mut input = String::new();
    stdout
        .execute(ScrollUp(N_LINES + 1))?
        .execute(MoveToPreviousLine(N_LINES + 1))?
        .execute(Print(PROMPT))?
        .flush()?;

    // Nodes in consideration
    let mut considered = vec![&root];
    // To be swapped with `considered` after every char input
    let mut new = vec![];
    loop {
        let Event::Key(event) = event::read()? else {
            continue;
        };

        let ch = match event.code {
            KeyCode::Char(ch) => {
                if event.modifiers == KeyModifiers::CONTROL && ch == 'c' {
                    break;
                } else if event.modifiers.contains(KeyModifiers::SHIFT) {
                    ch.to_uppercase().next().unwrap()
                } else {
                    ch
                }
            }
            KeyCode::Backspace => {
                stdout
                    .execute(MoveToColumn(PROMPT.len() as u16))?
                    .execute(Clear(UntilNewLine))?
                    .flush()?;
                input.clear();
                considered.clear();
                considered.push(&root);
                continue;
            }
            KeyCode::Esc => break,
            _ => continue,
        };

        input.push(ch);
        let misclicks = qwerty_misclicks(ch);
        new.clear();
        new.extend(
            considered.iter().flat_map(|n| n.get(ch)).chain(
                considered
                    .iter()
                    .flat_map(|n| misclicks.clone().filter_map(|ch| n.get(ch))),
            ),
        );
        swap(&mut new, &mut considered);

        stdout
            .execute(Clear(CurrentLine))?
            .execute(Clear(FromCursorDown))?
            .execute(MoveToColumn(0))?
            .execute(Print(PROMPT))?
            .execute(Print(&input))?
            .execute(MoveToNextLine(1))?;
        let mut lines_printed = 0;
        for node in &considered {
            node.for_each::<io::Error>(|i| {
                if lines_printed < N_LINES {
                    lines_printed += 1;
                    stdout
                        .execute(Print(&lines[i]))?
                        .execute(MoveToNextLine(1))?;
                }
                Ok(())
            })?;
        }
        stdout
            .execute(MoveToPreviousLine(lines_printed + 1))?
            .execute(MoveToColumn((PROMPT.len() + input.len()) as u16))?
            .flush()?;
    }

    Ok(())
}

fn main() -> ExitCode {
    let res = inner_main();
    _ = disable_raw_mode();
    match res {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
