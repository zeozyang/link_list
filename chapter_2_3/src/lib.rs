fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

#[derive(Debug)]
pub struct List<T> {
    head: Link<T>,
}

///使用type关键字声明类型的别名。
type Link<T> = Option<Box<Node<T>>>;

#[derive(Debug)]
struct Node<T> {
    elem: T,
    next: Link<T>,
}

///元组结构体
#[derive(Debug)]
pub struct IntoIter<T> (List<T>);

pub struct Iter<'a, T> (Option<&'a Node<T>>);

pub struct IterMut<'a, T> (Option<&'a mut Node<T>>);

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            head: None
        }
    }

    ///在链表头加一个节点
    pub fn push(&mut self, elem: T) {
        let node = Box::new(Node {
            elem,
            next: self.head.take(), //点运算符将执行很多类型转换的魔法：它将执行自动引用、自动去引用和强制转换，直到类型匹配。
        });
        self.head = Some(node);
    }
    ///弹出链表头节点，并获取其中的值
    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next; //这里可以像用Node类型一样用Box<Node>,是因为Box<T>实现了隐式Deref转换
            node.elem
        })
    }

    ///查看链表头节点的值，以引用的方式
    pub fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|node| {
            &node.elem
        })
    }
    ///查看链表头节点的值，以可变引用的方式
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| {
            &mut node.elem
        })
    }

    ///into_iter会夺走所有权
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self) //元组结构体方式的new函数
    }
    ///iter是借用
    pub fn iter(&self) -> Iter<T> {
        Iter(self.head.as_deref())
    }
    ///iter_mut是可变借用
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut(self.head.as_deref_mut())
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut link = self.head.take();
        while let Some(mut node) = link {
            link = node.next.take();
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.map(|node| { //self.0的类型是是Option<&>:
            // & is copy, Option<&> is also Copy. So when we did self.0.map
            // it was fine because the Option was just copied.
            self.0 = node.next.as_deref();
            &node.elem
        })
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.take().map(|node| { //&mut isn't Copy (if you copied an &mut,
            // you'd have two &mut's to the same location in memory, which is forbidden).
            // Instead, we should properly take the Option to get it.
            self.0 = node.next.as_deref_mut();
            &mut node.elem
        })
    }
}

#[cfg(test)]
mod tests {
    use super::List;

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.peek(), Some(&3));

        //rust圣经：手动迭代必须将迭代器声明为mut可变。
        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
        println!("{:?}", iter);
    }

    #[test]
    fn iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_mut() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter_mut = list.iter_mut();
        assert_eq!(iter_mut.next(), Some(&mut 3));
        assert_eq!(iter_mut.next(), Some(&mut 2));
        assert_eq!(iter_mut.next(), Some(&mut 1));
        assert_eq!(iter_mut.next(), None);
    }
}
