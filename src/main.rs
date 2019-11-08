use neovim_lib::*;
use rusqlite::*;

fn main() {
    let mut event_handler = EventHandler::new();

    event_handler.recv();
}

struct Discusser {
    connection: Connection,
}

const COMMENT_DB: &'static str = ".comment_code.db";

impl Discusser {
    fn init_connection() -> Result<Connection> {
        let connection = Connection::open(COMMENT_DB)?;

        connection.execute(
            "create table if not exists code_comments (
                id integer primary key,
                file_name text,
                start integer,
                end integer,
                comment text
            )",
            NO_PARAMS,
        )?;
        Ok(connection)
    }

    fn new() -> Result<Self> {
        let connection = Discusser::init_connection()?;
        Ok(Self { connection })
    }

    fn add_comment(&self, file_name: &str, start: i64, end: i64, comment: &str) -> Result<()> {
        self.connection.execute(
            "insert into code_comments (file_name, start, end, comment) values (?1, ?2, ?3, ?4)",
            &[file_name, &start.to_string(), &end.to_string(), comment],
        )?;
        Ok(())
    }

    fn get_comment(&self, file_name: &str, line: i64) -> Result<Option<String>> {
        let mut ranges_query = self.connection.prepare(
            "select comment from code_comments where file_name = ?1 and start <= ?2 and end >=?2 limit 1;",
        )?;

        let comment: Option<Result<String>> = ranges_query
            .query_map(&[file_name, &line.to_string()], |tuple| tuple.get(0))?
            .next();

        if let Some(comment) = comment {
            Ok(Some(comment?))
        } else {
            Ok(None)
        }
    }

    fn delete_comment(&self, file_name: &str, line: i64) -> Result<Vec<(i64, i64)>> {
        let mut ranges_query = self.connection.prepare(
            "select start, end from code_comments where file_name = ?1 and start <= ?2 and end >=?2;",
        )?;

        let line_name = line.to_string();

        let result: Result<Vec<(i64, i64)>> = ranges_query
            .query_map(&[file_name, &line_name], |tuple| {
                Ok((tuple.get(0)?, tuple.get(1)?))
            })?
            .collect();

        self.connection.execute(
            "delete from code_comments where file_name = ?1 and start <= ?2 and end >= ?2",
            &[file_name, &line_name],
        )?;

        result
    }

    fn get_ranges_in_file(&mut self, file_name: &str) -> Result<Vec<(i64, i64)>> {
        let mut ranges_query = self
            .connection
            .prepare("select start, end from code_comments where file_name = ?1;")?;

        let result: Result<Vec<(i64, i64)>> = ranges_query
            .query_map(&[file_name], |tuple| Ok((tuple.get(0)?, tuple.get(1)?)))?
            .collect();
        result
    }
}

struct EventHandler {
    nvim_instance: Neovim,
    discusser: Discusser,
}

impl EventHandler {
    fn new() -> Self {
        let session = Session::new_parent().unwrap();
        let nvim_instance = Neovim::new(session);
        let discusser = Discusser::new().unwrap();

        Self {
            nvim_instance,
            discusser,
        }
    }

    fn highlight_comment(&mut self, file_name: &str, start_line_num: i64, end_line_num: i64) {
        self.nvim_instance
            .command(&format!(
                "sign place {0} line={0} name=annotation file={1}",
                start_line_num, file_name
            ))
            .unwrap();
        for line_num in start_line_num + 1..=end_line_num {
            self.nvim_instance
                .command(&format!(
                    "sign place {0} line={0} name=annotationContinued file={1}",
                    line_num, file_name
                ))
                .unwrap();
        }
    }

    fn delete_highlight(&mut self, file_name: &str, start_line_num: i64, end_line_num: i64) {
        for line_num in start_line_num..=end_line_num {
            self.nvim_instance
                .command(&format!("sign unplace {} file={}", line_num, file_name))
                .unwrap_or_else(|_| {
                    self.nvim_instance
                        .command("echoerr \"Cannot unplace sign\"")
                        .unwrap();
                });
        }
    }

    fn recv(&mut self) {
        let receiver = self.nvim_instance.session.start_event_loop_channel();

        for (event, vals) in receiver {
            match Message::from(event) {
                Message::NewComment => {
                    if vals.len() >= 4 {
                        let file_name = vals[0].as_str().unwrap();
                        let (start_line_num, end_line_num) =
                            (vals[1].as_i64().unwrap(), vals[2].as_i64().unwrap());
                        self.discusser
                            .add_comment(
                                file_name,
                                start_line_num,
                                end_line_num,
                                vals[3].as_str().unwrap(),
                            )
                            .unwrap();
                        self.highlight_comment(file_name, start_line_num, end_line_num);
                    }

                    self.nvim_instance
                        .command("echom \"A new comment?!\"")
                        .unwrap()
                }
                Message::HighlightComments => {
                    if vals.len() >= 1 {
                        let file_name = vals[0].as_str().unwrap();
                        let ranges = self.discusser.get_ranges_in_file(file_name).unwrap();
                        for (start_line_num, end_line_num) in ranges {
                            self.highlight_comment(file_name, start_line_num, end_line_num);
                        }
                    }
                }
                Message::ShowComment => {
                    if vals.len() >= 2 {
                        let file_name = vals[0].as_str().unwrap();
                        let line_num = vals[1].as_i64().unwrap();
                        if let Some(comment) =
                            self.discusser.get_comment(file_name, line_num).unwrap()
                        {
                            self.nvim_instance
                                .command(&format!(
                                    "call discuss_code#Display_comment(\"{}\")",
                                    comment
                                ))
                                .unwrap()
                        }
                    }
                }
                Message::DeleteComment => {
                    if vals.len() >= 2 {
                        let file_name = vals[0].as_str().unwrap();
                        let line_num = vals[1].as_i64().unwrap();
                        let lines = self
                            .discusser
                            .delete_comment(file_name, line_num)
                            .unwrap_or_else(|_| {
                                self.nvim_instance
                                    .command("echoerr \"Cannot unplace sign\"")
                                    .unwrap();
                                Vec::new()
                            });
                        for (start, end) in lines {
                            self.delete_highlight(file_name, start, end);
                        }
                    }

                    self.nvim_instance
                        .command("echom \"Bye little friend\"")
                        .unwrap()
                }
                Message::Unknown(event) => self
                    .nvim_instance
                    .command(&format!("echo \"Unknown command: {}\"", event))
                    .unwrap(),
            }
        }
    }
}

enum Message {
    NewComment,
    ShowComment,
    DeleteComment,
    HighlightComments,
    Unknown(String),
}

impl From<String> for Message {
    fn from(event: String) -> Self {
        match event.as_str() {
            "new_comment" => Message::NewComment,
            "show_comment" => Message::ShowComment,
            "highlight_comments" => Message::HighlightComments,
            "delete_comment" => Message::DeleteComment,
            _ => Message::Unknown(event),
        }
    }
}
