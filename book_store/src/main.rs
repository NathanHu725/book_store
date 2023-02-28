use std::{env, io, thread, process};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};
use xml_rpc::server::*;
use xml_rpc::Fault;
use rand::Rng;
use sqlite;
use sqlite::State;

static db_name: &'static str = "bookstore.db";

fn restock() -> String {
    let connection = sqlite::open(db_name).unwrap();
    let new_stock = rand::thread_rng().gen_range(5..25);

    let query = "UPDATE BOOKS SET stock = stock + ".to_owned() + &new_stock.to_string();

    connection.execute(query).unwrap();
    format!("Successfully restocked {} books for each category!", new_stock)
}

fn buy(item_number: i64) -> Result<String, Fault> {
    let connection = sqlite::open(db_name).unwrap();
    let mut query = "SELECT stock FROM BOOKS WHERE item_number = ?";

    let mut statement = connection.prepare(query).unwrap();
    statement.bind((1, item_number)).unwrap();

    let stock_num = statement.read::<i64, _>("stock").unwrap();

    if stock_num <= 0 {
        return Err(Fault::new(400, String::from("No remaining stock of {item_number}")));
    };

    query = "UPDATE BOOKS SET stock = stock - 1 WHERE item_number == ?";
    connection.prepare(query).unwrap().bind((1, item_number)).unwrap();
    Ok(String::from("Bought 1 copy of {item_number}\n"))
}

fn lookup(subject: String) -> Result<String, Fault> {
    let connection = sqlite::open(db_name).unwrap();
    let mut query = "SELECT * FROM BOOKS WHERE subject = ?";

    let mut statement = connection.prepare(query).unwrap();
    statement.bind((1, subject.as_str())).unwrap();

    let mut matching_books = String::new();

    while let Ok(State::Row) = statement.next() {
        let book_info = format!(
            "Title: {} | ID: {} | Subject: {} | Cost {}\n",
            statement.read::<String, _>("title").unwrap(),
            statement.read::<String, _>("item_number").unwrap(),
            statement.read::<String, _>("subject").unwrap(),
            statement.read::<f64, _>("cost").unwrap(),
        );
        matching_books += book_info.as_str();
    }

    Ok(matching_books)
}

fn search(item_number: i64) -> Result<String, Fault> {
    let connection = sqlite::open(db_name).unwrap();
    let mut query = "SELECT * FROM BOOKS WHERE item_number = ?";

    let mut statement = connection.prepare(query).unwrap();
    statement.bind((1, item_number)).unwrap();

    let book_info = format!(
        "Title: {} | ID: {} | Subject: {} | Cost {}",
        statement.read::<String, _>("title").unwrap(),
        statement.read::<String, _>("item_number").unwrap(),
        statement.read::<String, _>("subject").unwrap(),
        statement.read::<f64, _>("cost").unwrap(),
    );

    Ok(book_info)
}

fn setup_server() {
    let args: Vec<String> = env::args().collect();

    let server_name: String = match args.get(1) {
        Some(a) => a.to_string() + &".cs.williams.edu:8013".to_string(),
        None => panic!("Enter a server name!"),
    };

    let ip_addr: SocketAddr = match server_name.to_socket_addrs().expect("Bad Server Name").next() {
        Some(a) => a,
        None => panic!("Server did not match an ip addr!"),
    };

    println!("{:?}", ip_addr);

    let ip_addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(137, 165, 161, 28)), 8013);

    thread::spawn(move || {
        let mut server: Server = Server::new();
        server.register_simple("buy", buy);
        server.register_simple("lookup", lookup);
        server.register_simple("search", search);

        let bound_server = match server.bind(&ip_addr2){
            Ok(a) => a,
            Err(e) => panic!("Err binding {}", e),
        };

        bound_server.run();
    });

}

fn main() {
    setup_server();

    println!("Now listening. Valid commands: restock, search, lookup, exit");
    let mut answer = String::from("");
    
    loop {
        io::stdin().read_line(&mut answer).unwrap();
        let parsed_answer = match answer.split("\r").next() {
            Some(a) => a,
            None => "error",
        };

        let response =  match parsed_answer {
            "restock" => Ok(restock()),
            "search" => {
                println!("Please enter the id of the book you'd like to search:");
                io::stdin().read_line(&mut answer).unwrap();
                match answer.split("\r").next() {
                    Some(a) => search(a.parse::<i64>().unwrap()),
                    None => Ok(String::from("error")),
                }
            },
            "lookup" => {
                println!("Please enter the subject of the book you'd like to see:");
                io::stdin().read_line(&mut answer).unwrap();
                match answer.split("\r").next() {
                    Some(a) => lookup(String::from(a)),
                    None => Ok(String::from("error")),
                }
            },
            "exit" => {
                process::exit(1);
            },
            _ => Ok(String::from("Invalid Command, please try again")),
        };
        println!("{}", match response {
            Ok(a) => a,
            Err(_) => String::from("Back end error"),
        })
    }
}
