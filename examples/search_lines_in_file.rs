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
    permissive_search::{SearchTree, Searcher},
    std::{
        env::args_os,
        error::Error,
        fs::File,
        io::{self, BufRead, BufReader, Write, stdout},
        ops::Deref,
        process::ExitCode,
    },
};

const PROMPT: &str = "> ";
const N_LINES: u16 = 10;

fn inner_main() -> Result<(), Box<dyn Error>> {
    let filename = args_os()
        .nth(1)
        .ok_or("Please provide the name of the file to read lines from")?;

    let file = File::open(&filename).map_err(|e| format!("Failed to open {filename:?}: {e}"))?;
    let lines = BufReader::new(file)
        .lines()
        .collect::<io::Result<Vec<_>>>()
        .map_err(|e| format!("Failed to read the contents of {filename:?}: {e}"))?;

    let root: SearchTree = lines.iter().map(Deref::deref).enumerate().collect();
    let mut searcher = Searcher::new(&root);

    enable_raw_mode()?;
    let mut stdout = stdout().lock();
    stdout
        .execute(ScrollUp(N_LINES + 1))?
        .execute(MoveToPreviousLine(N_LINES + 1))?
        .execute(Print(PROMPT))?
        .flush()?;

    loop {
        let Event::Key(event) = event::read()? else {
            continue;
        };

        match event.code {
            KeyCode::Char(ch) => {
                if event.modifiers == KeyModifiers::CONTROL && ch == 'c' {
                    break;
                } else if event.modifiers.contains(KeyModifiers::SHIFT) {
                    searcher.extend(ch.to_uppercase());
                } else {
                    searcher.push(ch);
                }
            }
            KeyCode::Backspace => {
                stdout
                    .execute(MoveToColumn(
                        (PROMPT.len() + searcher.input().len()) as u16 - 1,
                    ))?
                    .execute(Clear(UntilNewLine))?
                    .flush()?;
                searcher.pop();
            }
            KeyCode::Esc => break,
            _ => continue,
        };

        stdout
            .execute(Clear(CurrentLine))?
            .execute(Clear(FromCursorDown))?
            .execute(MoveToColumn(0))?
            .execute(Print(PROMPT))?
            .execute(Print(searcher.input()))?
            .execute(MoveToNextLine(1))?;
        let mut lines_printed = 0;
        searcher.for_each_candidate::<io::Error>(|i| {
            if lines_printed < N_LINES {
                lines_printed += 1;
                stdout
                    .execute(Print(&lines[i]))?
                    .execute(MoveToNextLine(1))?;
            }
            Ok(())
        })?;
        stdout
            .execute(MoveToPreviousLine(lines_printed + 1))?
            .execute(MoveToColumn((PROMPT.len() + searcher.input().len()) as u16))?
            .flush()?;
    }

    Ok(())
}

fn main() -> ExitCode {
    let res = inner_main();
    _ = disable_raw_mode();
    match res {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
