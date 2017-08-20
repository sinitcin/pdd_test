extern crate clap;
extern crate serde_json;

use clap::{Arg, App};
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::net::TcpStream;
use std::slice;
use std::str;
use serde_json::{Value};

type Result<T> = std::result::Result<T, String>;

fn err_interactive(buffer: &str, text_cmd: &str) -> Result<bool> {
// Диалог при ошибке о продолжении работы 

    let v: Value;
    match serde_json::from_str(&buffer) {
        Ok(expr) => v = expr,
        Err(_) => return Err(format!("Не смог разобрать ответ от команды \"{}\". Структура JSON формата не корректна...", text_cmd).to_owned()),
    }    
    let code;
    match v["code"].as_i64() {
        Some(expr) => code = expr,
        None => return Err(format!("Не смог разобрать ответ от команды \"{}\". Не могу найти поле \"code\" в структуре JSON формата...\n\n{}", text_cmd, buffer).to_owned()),
    }
    if vec![200, 201, 202, 203, 204].into_iter().find(|&x| x == code) != Some(code) {    
        println!("===================================================================");
        println!("=      Произошла фатальная ошибка!!! Продолжить выполнение?       =");
        println!("===================================================================");
        println!("Ответ на команду: \"{}\"\n\tсодержит ошибку {}.", &text_cmd, code);
        println!("===================================================================");
        print!("Введите да или нет: ");
        let _ = io::stdout().flush();
        loop {            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            match input.to_lowercase().trim() {
                "да" | "yes" | "y" => break,
                "нет" | "no" | "n" => return Err(format!("Тест не пройден, ошибка в запросе {} ", &text_cmd).to_owned()),
                _ => {
                    println!("Вы ввели не корректное значение, попробуйте снова: ");
                    continue;
                },
            }
        }
    }
    Ok(true)
}

fn telnet(host: &str, text_cmd: &str) -> Result<bool> {
// Эмуляция телнета
    
    // Преобразуем строку в массив байт для отправки
    let ptr = text_cmd.as_ptr();
    let len = text_cmd.len();
    let command = unsafe {    
        let slice = slice::from_raw_parts(ptr, len);
        slice
    };

    // Отправка команды 
    let mut stream;
    match TcpStream::connect(host) {
        Ok(expr) => stream = expr,
        Err(_) => return Err("Не могу подключиться к УСПД".to_owned()),
    }
    println!("\nОтправляем команду:\n{}", &text_cmd);
    let _ = stream.write_all(command);

    // Получение ответа
    let mut buffer = String::from("");
    let _ = stream.read_to_string(&mut buffer); 
    println!("\nПолучаем ответ:\n{}\n", &buffer);

    // Проверка результата
    let result = err_interactive(&buffer, &text_cmd)?;
    Ok(result)
}

fn main() {
    // Аргументы командной строки
    let matches = App::new("pdd_test")
        .version("1.0")
        .about("Приложение для тестирования корректной обработки текстового протокола поверх TCP!")
        .author("2017 (c) ЗАО НВП \"Болид\" - Синицын А. А.")
        .arg(Arg::with_name("host")
            .short("h")
            .long("host")
            .value_name("127.0.0.1:8080")
            .help("Установить IPv4 адрес и порт для подключения к серверу telnet.")
            .takes_value(true))
        .arg(Arg::with_name("file")
            .short("f")
            .long("file")
            .help("Указать путь к файлу с командами. Каждая строка в этом файле будет считаться отдельной командой.")
            .takes_value(true))
        .get_matches();

    // Получили параметры
    let host = matches.value_of("host").unwrap_or("127.0.0.1:8080");
    let file = matches.value_of("file").unwrap_or("commands.txt");
    println!("> Хост команд:\t{}\n> Файл команд:\t{}", host, file);

    // Чтение комманд из файла
    let f = File::open(file).unwrap();
    let reader = BufReader::new(f);
    for line in reader.lines() {
        match telnet(host, &line.unwrap()) {
            Ok(expr) => if expr {
                            println!("> Команда выполнена успешно");
                        } else {
                            println!("> Произошла ошибка при выполнении команды");
                        },
            Err(expr) => panic!("{}", expr),
        };
    }

    // Завершаем работу
    println!("> Все тесты пройдены успешно!!!");
}