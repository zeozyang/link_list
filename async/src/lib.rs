#[cfg(test)]
mod thread {
    use std::thread;
    use std::time::Duration;

    ///# 创建线程
    /// 使用thread::spawn可以创建线程:
    ///
    /// 线程内部的代码使用闭包来执行
    ///
    /// main线程一旦结束，程序就立刻结束，因此需要保持它的存活，直到其它子线程完成自己的任务
    ///
    /// thread::sleep会让当前线程休眠指定的时间，随后其它线程会被调度运行(上一节并发与并行中有简单介绍过)，因此就算你的电脑只有一个CPU核心，该程序也会表现的如同多CPU核心一般，这就是并发！
    #[test]
    fn creat_thread() {
        let handle = thread::spawn(|| {
            for i in 1..5 {
                println!("hi number {} from the spawned thread!", i);
                thread::sleep(Duration::from_millis(1));
            }
        });

        handle.join().unwrap();

        for i in 1..5 {
            println!("hi number {} from the main thread!", i);
            thread::sleep(Duration::from_millis(1));
        }
    }

    ///## 使用move关键字
    /// 将闭包所捕获的变量的所有权从一个线程转移到另外一个线程。
    #[test]
    fn _move() {
        let v = vec![1, 2, 3];

        let handle = thread::spawn(move || {
            println!("Here's a vector: {:?}", v);
        });

        handle.join().unwrap();
        // 下面代码会报错borrow of moved value: `v`
        // println!("{:?}",v);
    }


    use std::sync::mpsc;

    ///# 线程同步：消息传递
    /// 管道 Channel
    ///
    /// • Channel 包含：发送端、接收端
    ///
    /// • 调用发送端的方法，发送数据
    ///
    /// • 接收端会检查和接收到达的数据
    ///
    /// • 如果发送端、接收端中任意一端被丢弃了，那么 Channel 就“关闭”了
    ///
    ///# 创建 Channel
    /// • 使用 mpsc::channel 函数来创建 Channel
    ///
    /// – mpsc 表示 multiple producer, single consumer（多个生产者、一个消费者）
    ///
    /// – 返回一个 tuple（元组）：里面元素分别是发送端、接收端
    #[test]
    fn channel() {
        // 创建一个消息通道, 返回一个元组：(发送者，接收者)
        let (tx, rx) = mpsc::channel();

        // 创建线程，并发送消息
        thread::spawn(move || {
            // 发送一个数字1, send方法返回Result<T,E>，通过unwrap进行快速错误处理
            tx.send(1).unwrap(); //.sent()会取得所发送变量的所有权，但此处i32实现了copy。
        });

        // 在主线程中接收子线程发送的消息并输出
        println!("receive {}", rx.recv().unwrap());
        // tx,rx对应发送者和接收者，它们的类型由编译器自动推导:
        //    tx.send(1)发送了整数，因此它们分别是mpsc::Sender<i32>和mpsc::Receiver<i32>类型，
        //    需要注意，由于内部是泛型实现，一旦类型被推导确定，该通道就只能传递对应类型的值, 例如此例中非i32类型的值将导致编译错误
        // 接收消息的操作rx.recv()会阻塞当前线程，直到读取到值，或者通道被关闭
        // 需要使用 move 将 tx 的所有权转移到子线程的闭包中
    }

    ///##使用for进行循环接收
    #[test]
    fn _for_rx() {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let vals = vec![
                String::from("hi"),
                String::from("from"),
                String::from("the"),
                String::from("thread"),
            ];

            for val in vals {
                tx.send(val).unwrap();
                thread::sleep(Duration::from_secs(1));
            }
        });

        //当子线程运行完成时，发送者tx会随之被drop，此时for循环将被终止，最终main线程成功结束。
        for received in rx {
            println!("Got: {}", received);
        }
    }

    ///## 使用多发送者
    #[test]
    fn multiple_tx() {
        let (tx, rx) = mpsc::channel();
        let tx1 = tx.clone();
        thread::spawn(move || {
            tx.send(String::from("hi from raw tx")).unwrap();
        });

        thread::spawn(move || {
            tx1.send(String::from("hi from cloned tx")).unwrap();
        });

        for received in rx {
            println!("Got: {}", received);
        }
    }


    use std::sync::Mutex;

    ///# 线程同步：锁、Condvar和信号量
    ///https://course.rs/advance/concurrency-with-threads/sync1.html
    ///## 单线程中使用Mutex
    ///和Box类似，数据被Mutex所拥有，要访问内部的数据，需要使用方法m.lock()向m申请一个锁, 该方法会阻塞当前线程，直到获取到锁，因此当多个线程同时访问该数据时，只有一个线程能获取到锁，其它线程只能阻塞着等待，这样就保证了数据能被安全的修改！
    #[test]
    fn _mutex() {
        // 使用`Mutex`结构体的关联函数创建新的互斥锁实例
        let m = Mutex::new(5);

        {
            // 使用.lock()获取锁，然后deref为`m`的引用
            // lock返回的是Result
            let mut num = m.lock().unwrap();
            *num = 6;
            // 锁自动被drop。此线程释放锁
        }

        println!("m = {:?}", m);
    }

    use std::sync::Arc;

    ///## 多线程中使用Mutex
    /// Rc<T>/RefCell<T>用于单线程内部可变性， Arc<T>/Mutext<T>用于多线程内部可变性。
    #[test]
    fn _arc_mutex() {
        // 通过`Arc`实现`Mutex`的多所有权
        let counter = Arc::new(Mutex::new(0));
        let mut handles = vec![];

        for _ in 0..10 {
            let counter = Arc::clone(&counter);
            // 创建的同时执行子线程，并将`Mutex`的所有权拷贝传入到子线程中
            let handle = thread::spawn(move || {
                let mut num = counter.lock().unwrap();
                *num += 1;
            });
            handles.push(handle);
        }

        // 等待所有子线程完成
        for handle in handles {
            handle.join().unwrap();
        }
        // 输出最终的计数结果
        println!("Result: {}", *counter.lock().unwrap());
    }

    //# Send与Sync
    //实现Send的类型可以在线程间安全的传递其所有权
    //实现了Sync的类型可以在线程间安全的共享(通过引用)
    //这里还有一个潜在的依赖：一个类型要在线程间安全的共享的前提是，指向它的引用必须能在线程间传递。因为如果引用都不能被传递，我们就无法在多个线程间使用引用去访问同一个数据了。
    //由上可知，若类型T的引用&T是Send，则T是Sync。
}