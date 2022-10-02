use crate::calculator;
use std::io::{stdout, Write};

#[derive(Clone, Debug)]
pub struct Controller {
    output_format: OutputFormat,
}

#[derive(Copy, Clone, Debug)]
pub enum OutputFormat {
    Standard
}

impl Controller {
    pub fn new(
        output_format: OutputFormat,
    ) -> Self {
        stdout().flush().unwrap();
        

        Self {
            output_format,
        }
    }

    pub fn execute(&mut self, command: String) -> String {
        let result = self.execute_core(command);

        match result {
            Ok(Some(output)) => {
                output
            }
            Err(error) => match self.output_format {
                OutputFormat::Standard => {
                    error
                }
            },
            _ => String::new(),
        }
    }

    fn execute_core(&mut self, command: String) -> Result<Option<String>, String> {
        fn print_machi(
            tehai: &calculator::Tehai,
            shanten: i32,
            conditions: Vec<calculator::MachiCondition>,
        ) -> String {
            format!(
                "手牌：{}\n{}",
                tehai,
                if shanten == -1 {
                    format!("和了")
                } else {
                    let mut conditions_string = String::new();
                    for i in conditions {
                        conditions_string += &format!("\n{}", i);
                    }
                    format!(
                        "{}\n--------{}",
                        if shanten == 0 {
                            format!("聴牌")
                        } else {
                            format!("向聴：{}", shanten)
                        },
                        conditions_string
                    )
                }
            )
        }

        let tehai = calculator::Tehai::new(
            command,
        )?;

        let (shanten, conditions) = tehai.analyze()?;
        return Ok(Some(print_machi(
            &tehai,
            shanten,
            conditions,
        )));
            
    }
}
