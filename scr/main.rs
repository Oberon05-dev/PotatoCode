use crossterm::{execute, cursor::{MoveUp, MoveDown, MoveTo , MoveLeft, MoveRight, SavePosition, RestorePosition, MoveToColumn, MoveToNextLine, MoveToPreviousLine, self}};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType};
use crossterm::event::{read, Event, KeyCode, KeyEvent};
use std::io::{self, Read, stdout, Write};
use std::process::Command;
use std::fs::File;
use core::panic;


struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Could not disable raw mode")
    }
}

fn raw_mode(buf_raw: &mut [u8], buf: &mut String){

    // detection of keys
    while io::stdin().read(buf_raw).expect("Failed to read line") == 1 && buf_raw != [b'q'] && buf_raw != [b'i'] && buf_raw != [b's'] && buf_raw != [b'o']{}

    // quiting of the program
    if buf_raw == [b'q']{
        panic!()
    } 

    // entering into input mode
    else if buf_raw == [b'i']{
        input_mode(buf, buf_raw);
    }

    // opening the file
    else if buf_raw == [b'o'] {
        let filename = "/home/Oberon/textEditor/text_editor/plik.txt";  // name of the file

        // Loading of the file (brbr)
        buf.clear();
        let mut file = File::open(filename).expect("Cannot open file.");
        file.read_to_string(buf).expect("Cannot load file.");
        input_mode(buf, buf_raw);
    }

    // saving the file
    else
    {
        let mut file = File::create("plik.txt").expect("Cannot create file.");
        file.write_all(buf.as_bytes()).expect("Save error.");
    }
}

fn input_mode(buf: &mut String, buf_raw: &mut [u8]){
    // defining the cursor position
    let mut cursor_position = 0;
    let mut start_line = 0; 
    const PAGE_SIZE: usize = 100;
    let max_lines = buf.lines().count();
    print!("{}", max_lines);
    let mut end_line = start_line + PAGE_SIZE.min(max_lines);

    // clearing terminal for better visuals
    Command::new("clear").status().expect("failed to clear screen");

    // entering the alternate screen (idk why but yes)
    execute!(stdout(), EnterAlternateScreen).expect("cannot enter Input mode");

    // printing buffer 

    let lines: Vec<&str> = buf.lines().skip(start_line).take(PAGE_SIZE).collect();
    for line in lines {
        print!("{}\n\r", line);
    }

    // moving cursor into position 0 0
    execute!(stdout(), MoveTo(0, 0)).unwrap();

    // loop that interprating keybord into text and toher good shit stuff
    loop {
        match read() {
            Ok(Event::Key(KeyEvent { code, .. })) => {
                match code {

                    // exit input modes
                    KeyCode::Esc => {
                        break;
                    }

                    // moving arround text (arrows)
                    KeyCode::Up => {
                        execute!(stdout(), MoveUp(1)).unwrap();
                    }
                    KeyCode::Down => {
                        execute!(stdout(), MoveDown(1)).unwrap();
                    }
                    KeyCode::Right => {
                        if cursor_position < buf.len() {
                            if buf.chars().nth(cursor_position) == Some('\n')
                            {
                                execute!(stdout(), MoveToColumn(0), MoveDown(1)).unwrap();
                                cursor_position += 2;
                            }
                            else
                            {
                                execute!(stdout(), MoveRight(1)).unwrap();
                                cursor_position += 1;
                            }
                        }
                    }
                    KeyCode::Left => {
                        execute!(stdout(), MoveLeft(1)).unwrap();
                        if cursor_position > 0
                        {
                            if buf.chars().nth(cursor_position - 1) == Some('\r')
                            {
                                execute!(stdout(), MoveToPreviousLine(1), MoveRight((cursor_position - 2).try_into().unwrap())).unwrap();
                                cursor_position -= 2;
                            }
                            else
                            {
                                cursor_position -= 1;
                            }
                        }

                    }   
                    
                    // deleting the text 
                    KeyCode::Backspace=> {
                        if !buf.is_empty() && cursor_position > 0 {
                            if buf.chars().nth(cursor_position - 1) == Some('\n')
                            {
                                buf.remove(cursor_position - 1);
                                cursor_position -= 1;
                                buf.remove(cursor_position - 1);
                                cursor_position -= 1;
                                // move cursor to left and delete text
                                execute!(stdout(), MoveLeft(2), Clear(ClearType::UntilNewLine)).unwrap();
                                print!("\x1B[2J\x1B[1;1H{}", buf);
                                stdout().flush().unwrap();
                            }
                            else if cursor_position >= 4 && buf.get(cursor_position - 4..cursor_position) == Some("    "){
                                for _ in 0..4 {
                                    if cursor_position > 0 {
                                        cursor_position -= 1;
                                        buf.remove(cursor_position);
                                    }
                                }
                                print!("\x1B[2J\x1B[1;1H{}", buf);
                                stdout().flush().unwrap();
                                execute!(stdout(), MoveLeft(3), SavePosition, Clear(ClearType::UntilNewLine)).expect("cannot remove tab");
                                print!("\x1B[2J\x1B[1;1H{}", buf);
                                stdout().flush().unwrap();
                                execute!(stdout(), RestorePosition).unwrap();
                            }
                            else
                            {
                                buf.remove(cursor_position - 1);
                                cursor_position -= 1;
                                // move cursor to left and delete text
                                execute!(stdout(), MoveLeft(1), SavePosition, Clear(ClearType::UntilNewLine)).unwrap();
                                print!("\x1B[2J\x1B[1;1H{}", buf);
                                stdout().flush().unwrap();
                                execute!(stdout(), RestorePosition).unwrap();
                            }
                        }
                    }

                    // new line
                    KeyCode::Enter => {
                        buf.push_str("\n\r");
                        cursor_position += 2;
                        execute!(stdout(), MoveToNextLine(1), MoveToColumn(0)).unwrap();
                    }

                    // make tab
                    KeyCode::Tab => {
                        buf.insert_str(cursor_position, "    ");
                        cursor_position += 4;
                        execute!(stdout(), MoveRight(4)).unwrap();
                        print!("\x1B[s\x1B[2J\x1B[1;1H{}\x1B[u", buf);
                        stdout().flush().unwrap();
                    }

                    // keys into text
                    KeyCode::Char(c) => {
                        buf.insert(cursor_position, c);
                        cursor_position += 1;
                        execute!(stdout(), MoveRight(1)).unwrap();
                        print!("\x1B[s\x1B[2J\x1B[1;1H{}\x1B[u", buf);
                        stdout().flush().unwrap();
                    }
                    _ => {}
                }
            }
            Ok(_) => {}
            Err(err) => {
                println!("Error reading input: {}", err);
                break;
            }
        }
    }
    execute!(stdout(), LeaveAlternateScreen).expect("cannot Leave Input mode");
    raw_mode(buf_raw, buf);
}

fn main() {
    let _clean_up = CleanUp;
    // enable raw mode in trminal
    terminal::enable_raw_mode().expect("Could not turn on Raw mode");
    // clear terminal
    Command::new("clear").status().expect("failed to clear screen");
    // buffer to cache inputs (text)
    let mut buf = String::new();
    // list for commands in raw mode
    let mut buf_raw = [0;1];
    // other nice stuff
    input_mode(&mut buf, &mut buf_raw);
}
