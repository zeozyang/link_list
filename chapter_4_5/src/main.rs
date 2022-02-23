use std::rc::Rc;
use std::cell::{Ref, RefMut, RefCell};

///# 使用 RefCell<T> 在运行时记录借用信息
///
/// • 两个方法（安全接口）：
///
///     – borrow 方法
///         • 返回智能指针 Ref<T>，它实现了 Deref
///     – borrow_mut 方法
///         • 返回智能指针 RefMut<T>，它实现了 Deref
///
/// • RefCell<T> 会记录当前存在多少个活跃的 Ref<T> 和 RefMut<T> 智能指针：
///
///     – 每次调用 borrow：不可变借用计数加 1
///     – 任何一个 Ref<T> 的值离开作用域被释放时：不可变借用计数减 1
///     – 每次调用 borrow_mut：可变借用计数加 1
///     – 任何一个 RefMut<T> 的值离开作用域被释放时：可变借用计数减 1
///
/// • 以此技术来维护借用检查规则：
///
///     – 任何一个给定时间里，只允许拥有多个不可变借用或一个可变借用。

pub struct List<T> {
    head: Link<T>,
    tail: Link<T>,
}

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
    prev: Link<T>,
}

impl<T> Node<T> {
    fn new(elem: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            elem,
            prev: None,
            next: None,
        }))
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            head: None,
            tail: None,
        }
    }

    pub fn push_front(&mut self, elem: T) {
        let node = Node::new(elem);
        match self.head.take() {
            Some(head) => {
                head.borrow_mut().prev = Some(node.clone()); //头节点的上个节点指向新节点
                node.borrow_mut().next = Some(head); //新节点的下个节点指向头节点
                self.head = Some(node);
            }
            None => {
                self.tail = Some(node.clone());
                self.head = Some(node);
            }
        }
    }

    pub fn push_back(&mut self, elem: T) {
        let node = Node::new(elem);
        match self.tail.take() {
            Some(tail) => {
                tail.borrow_mut().next = Some(node.clone());
                node.borrow_mut().prev = Some(tail);
                self.tail = Some(node);
            }
            None => {
                self.head = Some(node.clone());
                self.tail = Some(node);
            }
        }
    }

    pub fn pop_front(&mut self) -> Option<T> { //                           List -> Node <=> Next
        self.head.take().map(|node| { //取出头节点node,  List    Node <=> Next
            match node.borrow_mut().next.take() { //取出头节点的下一个节点,     List    Node <-  Next
                Some(next) => {
                    next.borrow_mut().prev.take(); //                       List    Node    Next
                    self.head = Some(next); //                              List ->  Next
                }
                None => {
                    self.tail.take();
                }
            }
            Rc::try_unwrap(node) //rc变result。Returns the inner value, if the Rc has exactly one strong reference.
                .ok()           //result变option。Converts from Result<T, E> to Option<T>.
                .unwrap()       //拆包option，得到refcell。在确认Option不为None的情况下，可以用unwrap方法拆解出其中的值，并获取值的所有权。
                .into_inner()   //拆包refcell，得到node。Consumes the RefCell, returning the wrapped value.
                .elem           //node.elem
        })
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.take().map(|node| {
            match node.borrow_mut().prev.take() {
                Some(prev) => {
                    prev.borrow_mut().next.take();
                    self.tail = Some(prev);
                }
                None => {
                    self.head.take();
                }
            }
            Rc::try_unwrap(node).ok().unwrap().into_inner().elem
        })
    }

    pub fn peek_front(&self) -> Option<Ref<T>> {
        self.head.as_ref().map(|node| {
            Ref::map(node.borrow(), |node| &node.elem)
        })
    }

    pub fn peek_back(&self) -> Option<Ref<T>> {
        self.tail.as_ref().map(|node| {
            Ref::map(node.borrow(), |node| &node.elem)
        })
    }

    pub fn peek_front_mut(&mut self) -> Option<RefMut<T>> {
        self.head.as_ref().map(|node| {
            RefMut::map(node.borrow_mut(), |node| &mut node.elem)
        })
    }

    pub fn peek_back_mut(&mut self) -> Option<RefMut<T>> {
        self.tail.as_ref().map(|node| {
            RefMut::map(node.borrow_mut(), |node| &mut node.elem)
        })
    }
}

//实现迭代器
//Iter 不实现
//IterMut 不实现
//IntoIter
pub struct IntoIter<T> (List<T>);

impl<T> List<T> {
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.0.pop_back()
    }
}


#[cfg(test)]
mod tests {
    use super::List;

    #[test]
    fn map() {
        use std::collections::HashMap;

        let mut scores = HashMap::new();

        scores.insert(String::from("Blue"), 10);
        scores.insert(String::from("Yellow"), 50);

        let team_name = String::from("Blue");
        let score = scores.get(&team_name);

        for (key, value) in &scores {
            println!("{}: {}", key, value);
        }


        let mut scores = HashMap::new();

        scores.insert("Blue", 10);

        // 覆盖已有的值
        let old = scores.insert("Blue", 20);
        assert_eq!(old, Some(10));

        // 查询新插入的值
        let new = scores.get("Blue");
        assert_eq!(new, Some(&20));

        // 查询Yellow对应的值，若不存在则插入新值
        let v = scores.entry("Yellow").or_insert(5);
        assert_eq!(*v, 5); // 不存在，插入5

        // 查询Yellow对应的值，若不存在则插入新值
        let v = scores.entry("Yellow").or_insert(50);
        assert_eq!(*v, 5); // 已经存在，因此50没有插入
    }

    #[test]
    fn basics() {
        let mut list = List::new();
        assert_eq!(list.pop_front(), None);

        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), Some(2));

        list.push_front(4);
        list.push_front(5);

        assert_eq!(list.pop_front(), Some(5));
        assert_eq!(list.pop_front(), Some(4));
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);

        //----back-----
        assert_eq!(list.pop_back(), None);
        list.push_back(4);
        list.push_back(5);
        assert_eq!(list.pop_back(), Some(5));
        assert_eq!(list.pop_back(), Some(4));
        assert_eq!(list.pop_back(), None);
    }

    #[test]
    fn peek() {
        let mut list = List::new();
        assert!(list.peek_front().is_none());
        assert!(list.peek_back().is_none());
        assert!(list.peek_front_mut().is_none());
        assert!(list.peek_back_mut().is_none());

        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        assert_eq!(*list.peek_front().unwrap(), 3);
        assert_eq!(*list.peek_front_mut().unwrap(), 3);
        assert_eq!(*list.peek_back().unwrap(), 1);
        assert_eq!(*list.peek_back_mut().unwrap(), 1);
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next_back(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next(), None);
    }
}

use std::cell::Cell;

fn main() {
    let c = Cell::new("asdf");
    let one = c.get();
    c.set("qwer");
    let two = c.get();
    println!("{},{}", one, two);

    // code snipet 1
    // let x = Cell::new(1);
    // let y = &x;
    // let z = &x;
    // x.set(2);
    // y.set(3);
    // z.set(4);
    // println!("{}", x.get());

    // code snipet 2
    // let mut x = 1;
    // let y = &mut x;
    // let z = &mut x;
    // x = 2;
    // *y = 3;
    // *z = 4;
    // println!("{}", x);


    fn is_even(i: i32) -> bool {
        i % 2 == 0
    }
    //
    // fn retain_even(nums: &mut Vec<i32>) {
    //     let mut i = 0;
    //     for num in nums.iter().filter(|&num| {is_even(*num)}) {
    //         nums[i] = *num;
    //         i += 1;
    //     }
    //     nums.truncate(i);
    // }

    ///在Rust1.37版本中新增了两个非常实用的方法:
    ///Cell::from_mut, 该方法将&mut T转为 &Cell<T>
    ///Cell::as_slice_of_cells，该方法将 &Cell<[T]>转为 &[Cell<T>]
    fn retain_even(nums: &mut Vec<i32>) {
        let slice = Cell::from_mut(&mut nums[..]).as_slice_of_cells();

        let mut i = 0;
        for num in slice.iter().filter(|num| is_even(num.get())) {
            slice[i].set(num.get());
            i += 1;
        }

        nums.truncate(i);
    }

    let c = RefCell::new((5, 'b'));
    let b1: Ref<(u32, char)> = c.borrow();
    let b2: Ref<u32> = Ref::map(b1, |t| &t.0);
    assert_eq!(*b2, 5);
    println!("{:?}", c);
}