use std::io::{Error, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

// 封装从每个流中读取并将其写回的逻辑
fn handle_client_request(mut stream: TcpStream) -> Result<(), Error>{
    // 打印客户端请求地址和端口
    println!("new connection from: {}", stream.peer_addr()?);

    // 声明一个mutable的临时保存数据的缓冲区 bufferfer，并清零
    let mut buffer = [0; 512];
    // 无条件循环体，服务不停歇
    loop {
        // 读取请求流内容，返回已读取的数据的长度，这里使用 ? 运算符来处理调用中可能的错误
        let bytes_read = stream.read(&mut buffer)?;
        println!("echo: {0}", String::from_utf8_lossy(&buffer[..]));
        // 如果已到达流的末尾，并跳出循环
        if bytes_read == 0 {
            return Ok(());
        }
        // 切片并将数据写回流
        stream.write(&buffer[..bytes_read])?;
    }
}

fn main() {
    // 创建一个新的 tcp 套接字，用于监听来自客户端4000端口的请求
    let listener = TcpListener::bind("127.0.0.1:4000").expect("Could not bind");
    println!("server listening on port 4000");

    // 遍历已连接到服务器的流上的迭代器
    for stream in listener.incoming() {
        // 模式匹配流
        match stream {
            // 错误处理
            Err(error) => eprintln!("failed: {}", error),
            // 正确处理
            Ok(stream) => {
                // 线程收到一个调用函数handle_client_request的闭包
                thread::spawn(move || {
                    // 从作用域中读取变量，打印可能的错误
                    handle_client_request(stream).unwrap_or_else(|error| eprintln!("{:?}", error));
                });
            }
        }
    }
}
