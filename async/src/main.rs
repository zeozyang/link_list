// `block_on`会阻塞当前线程直到指定的`Future`执行完成，这种阻塞当前线程以等待任务完成的方式较为简单、粗暴，
// 好在其它运行时的执行器(executor)会提供更加复杂的行为，例如将多个`future`调度到同一个线程上执行。
use futures::executor::block_on;

/// # async编程入门
async fn hello_world() {
    // 在async fn函数中使用.await可以等待另一个异步调用的完成。
    // 但是与block_on不同，.await并不会阻塞当前的线程，而是异步的等待Future A的完成，(这时会开始运行Future A)
    // 在等待的过程中，该线程还可以继续执行其它的Future B，最终实现了并发处理的效果。
    hello_cat().await;
    println!("hello, world!");
}

async fn hello_cat() {
    println!("hello, kitty!");
}

fn main1() {
    let future = hello_world(); // 返回一个Future, 因此不会打印任何输出
    block_on(future); // 执行`Future`并等待其运行完成，此时"hello, world!"会被打印输出
}


async fn learn_song() -> Song { /* ... */ }

async fn sing_song(song: Song) { /* ... */ }

async fn dance() { /* ... */ }

async fn learn_and_sing() {
    // 这里使用`.await`来等待学歌的完成，但是并不会阻塞当前线程，该线程在学歌的任务`.await`后，完全可以去执行跳舞的任务
    let song = learn_song().await;

    // 唱歌必须要在学歌之后
    sing_song(song).await;
}

async fn async_main() {
    let f1 = learn_and_sing();
    let f2 = dance();

    // `join!`可以并发的处理和等待多个`Future`，
    // 若`learn_and_sing Future`被阻塞，那`dance Future`可以拿过线程的所有权继续执行。
    // 若`dance`也变成阻塞状态，那`learn_and_sing`又可以再次拿回线程所有权，继续执行。
    // 若两个都被阻塞，那么`async main`会变成阻塞状态，然后让出线程所有权，并将其交给`main`函数中的`block_on`执行器
    futures::join!(f1, f2);
}

fn main() {
    block_on(async_main());
}


///# 底层探秘: Future执行器与任务调度
/// 首先，来给出 Future 的定义：它是一个能产出值的异步计算(虽然该值可能为空，例如 () )。光看这个定义，可能会觉得很空洞，我们来看看一个简化版的 Future 特征:
trait SimpleFuture {
    type Output;
    fn poll(&mut self, wake: fn()) -> Poll<Self::Output>;
}

enum Poll<T> {
    Ready(T),
    Pending,
}

///考虑一个需要从 socket 读取数据的场景：如果有数据，可以直接读取数据并返回 Poll::Ready(data)， 但如果没有数据，Future 会被阻塞且不会再继续执行，此时它会注册一个 wake 函数，当 socket 数据准备好时，该函数将被调用以通知执行器：我们的 Future 已经准备好了，可以继续执行。
///
/// 下面的 SocketRead 结构体就是一个 Future:
pub struct SocketRead<'a> {
    socket: &'a Socket,
}

impl SimpleFuture for SocketRead<'_> {
    type Output = Vec<u8>;

    fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
        if self.socket.has_data_to_read() {
            // socket有数据，写入buffer中并返回
            Poll::Ready(self.socket.read_buf())
        } else {
            // socket中还没数据
            //
            // 注册一个`wake`函数，当数据可用时，该函数会被调用，
            // 然后当前Future的执行器会再次调用`poll`方法，此时就可以读取到数据
            self.socket.set_readable_callback(wake);
            Poll::Pending
        }
    }
}


///## 如何同时运行多个 Future
/// 一个SimpleFuture，它会并发地运行两个Future直到它们完成
///
/// 之所以可以并发，是因为两个Future的轮询可以交替进行，一个阻塞，另一个就可以立刻执行，反之亦然
pub struct Join<FutureA, FutureB> {
    // 结构体的每个字段都包含一个Future，可以运行直到完成.
    // 如果Future完成后，字段会被设置为 `None`. 这样Future完成后，就不会再被轮询
    a: Option<FutureA>,
    b: Option<FutureB>,
}

impl<FutureA, FutureB> SimpleFuture for Join<FutureA, FutureB>
    where
        FutureA: SimpleFuture<Output=()>,
        FutureB: SimpleFuture<Output=()>,
{
    type Output = ();
    fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
        // 尝试去完成一个 Future `a`
        if let Some(a) = &mut self.a {
            if let Poll::Ready(()) = a.poll(wake) {
                self.a.take();
            }
        }

        // 尝试去完成一个 Future `b`
        if let Some(b) = &mut self.b {
            if let Poll::Ready(()) = b.poll(wake) {
                self.b.take();
            }
        }

        if self.a.is_none() && self.b.is_none() {
            // 两个 Future都已完成 - 我们可以成功地返回了
            Poll::Ready(())
        } else {
            // 至少还有一个 Future 没有完成任务，因此返回 `Poll::Pending`.
            // 当该 Future 再次准备好时，通过调用`wake()`函数来继续执行
            Poll::Pending
        }
    }
}


///## 多个Future也可以一个接一个的连续运行
/// 一个SimpleFuture, 它使用顺序的方式，一个接一个地运行两个Future
///
/// 注意: 由于本例子用于演示，因此功能简单，`AndThenFut` 会假设两个 Future 在创建时就可用了.
/// 而真实的`Andthen`允许根据第一个`Future`的输出来创建第二个`Future`，因此复杂的多。
pub struct AndThenFut<FutureA, FutureB> {
    first: Option<FutureA>,
    second: FutureB,
}

impl<FutureA, FutureB> SimpleFuture for AndThenFut<FutureA, FutureB>
    where
        FutureA: SimpleFuture<Output=()>,
        FutureB: SimpleFuture<Output=()>,
{
    type Output = ();
    fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
        if let Some(first) = &mut self.first {
            match first.poll(wake) {
                // 我们已经完成了第一个 Future， 可以将它移除， 然后准备开始运行第二个
                Poll::Ready(()) => self.first.take(),
                // 第一个 Future 还不能完成
                Poll::Pending => return Poll::Pending,
            };
        }

        // 运行到这里，说明第一个Future已经完成，尝试去完成第二个
        self.second.poll(wake)
    }
}


///##真实的 Future 特征有何不同之处
/// - 首先这里多了一个 Pin ，关于它我们会在后面章节详细介绍，现在你只需要知道使用它可以创建一个无法被移动的 Future ，因为无法被移动，因此它将具有固定的内存地址，意味着我们可以存储它的指针(如果内存地址可能会变动，那存储指针地址将毫无意义！)，也意味着可以实现一个自引用数据结构: struct MyFut { a: i32, ptr_to_a: *const i32 }。 而对于 async/await 来说，Pin 是不可或缺的关键特性。
/// - 其次，从 wake: fn() 变成了 &mut Context<'_> 。意味着 wake 函数可以携带数据了，为何要携带数据？考虑一个真实世界的场景，一个复杂应用例如web服务器可能有数千连接同时在线，那么同时就有数千 Future 在被同时管理着，如果不能携带数据，当一个 Future 调用 wake 后，执行器该如何知道是哪个 Future 调用了 wake ,然后进一步去 poll 对应的 Future ？没有办法！那之前的例子为啥就可以使用没有携带数据的 wake ？ 因为足够简单，不存在歧义性。
/// - 总之，在正式场景要进行 wake ，就必须携带上数据。 而 Context 类型通过提供一个 Waker 类型的值，就可以用来唤醒特定的的任务。
trait Future {
    type Output;
    fn poll(
        // 首先值得注意的地方是，`self`的类型从`&mut self`变成了`Pin<&mut Self>`:
        self: Pin<&mut Self>,
        // 其次将`wake: fn()` 修改为 `cx: &mut Context<'_>`:
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output>;
}