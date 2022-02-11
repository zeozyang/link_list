#[cfg(test)]
mod un_safe {
    /// # Unsafe 超能力
    /// ## 使用 unsafe 关键字来切换到 unsafe Rust，开启一个块，里面放着 unsafe 代码
    ///
    /// • Unsafe Rust 里可执行的四个动作（unsafe 超能力）：
    ///
    ///     – 解引用原始指针
    ///     – 调用 unsafe 函数或方法
    ///     – 访问或修改可变的静态变量
    ///     – 实现 unsafe trait
    ///
    /// • 注意：
    ///
    ///     – unsafe 并没有关闭借用检查或停用其它安全检查
    ///     – 任何内存安全相关的错误必须留在 unsafe 块里
    ///     – 尽可能隔离 unsafe 代码，最好将其封装在安全的抽象里，提供安全的API

    /// # 解引用原始指针
    /// • 原始指针
    ///
    ///     – 可变的：*mut T
    ///     – 不可变的：*const T。意味着指针在解引用后不能直接对其进行赋值
    ///     – 注意：这里的 * 不是解引用符号，它是类型名的一部分。
    ///
    /// • 与引用不同，原始指针：
    ///
    ///     – 允许通过同时具有不可变和可变指针 或 多个指向同一位置的可变指针来忽略借用规则
    ///     – 无法保证能指向合理的内存
    ///     – 允许为 null
    ///     – 不实现任何自动清理
    ///
    /// • 放弃保证的安全，换取更好的性能/与其它语言或硬件接口的能力
    #[test]
    fn raw_pointer() {
        let mut num = 5;
        //可以在不安全代码块之外创建原始指针，但只能在不安全代码块之内对其解引用
        //这里使用 as 将不可变和可变引用强转为对应的裸指针类型。
        let r1 = &num as *const i32;
        let r2 = &mut num as *mut i32;
        unsafe {
            println!("r1 is: {}", *r1);
            println!("r2 is: {}", *r2);
        }
    }

    ///# 调用 unsafe 函数或方法
    /// • unsafe 函数或方法：在定义前加上了 unsafe 关键字
    ///
    ///     – 调用前需手动满足一些条件（主要靠看文档），因为 Rust 无法对这些条件进行验证
    ///     – 需要在 unsafe 块里进行调用
    #[test]
    fn call_unsafe() {
        unsafe fn dangerous() {}

        unsafe {
            dangerous();
        }
    }

    ///## 创建不安全代码的安全抽象
    #[test]
    fn call_safe() {
        let mut v = vec![1, 2, 3, 4, 5, 6];
        let r = &mut v[..];
        let (a, b) = r.split_at_mut(3);
        assert_eq!(a, &mut [1, 2, 3]);
        assert_eq!(b, &mut [4, 5, 6]);
    }

    ///## 使用 extern 函数调用外部代码
    /// • extern 关键字：简化创建和使用外部函数接口（FFI）的过程。
    ///
    /// • 外部函数接口（ FFI ，Foreign Function Interface）：它允许一种编程语言定义函数，并让其它编程语言能调用这些函数
    ///
    /// • 应用二进制接口（ABI，Application Binary Interface）：定义函数在汇编层的调用方式
    ///
    /// • “C”ABI是最常见的 ABI，它遵循 C 语言的 ABI
    #[test]
    fn extern_c() {
        unsafe {
            println!("Absolute value of -3 according to C: {}", abs(-3));
        }
    }

    extern "C" {
        fn abs(input: i32) -> i32;
    }

    ///## 从其它语言调用 Rust 函数
    /// • 可以使用 extern 创建接口，其它语言通过它们可以调用 Rust 的函数
    ///
    /// • 在 fn 前添加 extern 关键字，并指定 ABI
    ///
    /// • 还需添加 #[no_mangle] 注解：避免 Rust 在编译时改变它的名称
    #[test]
    fn extern_rust() {
        #[no_mangle]
        pub extern "C" fn call_from_c() {
            println!("Just called a Rust function from C!");
        }
    }
}


#[cfg(test)]
mod data_layout {
    use std::mem;

    ///# rust中的数据布局

    ///## 动态尺寸类型DST - slice
    #[test]
    fn slice_layout() {
        struct A<'a> {
            _a: i32,
            _b: &'a [u8], //_b是数组切片
        }
        let array = [1; 10];
        let s = &array[..]; //s是数组切片

        println!("s size = {}", mem::size_of_val(s));   //4 * 10 = 40 bytes
        println!("&s size = {}", mem::size_of_val(&s)); //8 * 2 = 16，&s是切片结构体本身，是个胖指针，里面有两个字段，一个是array的引用，一个是切片的长度
        println!("&i32 size = {}", mem::size_of::<&i32>()); //8，我是64位的电脑，所以引用的尺寸就是8bytes
        println!("&i64 size = {}", mem::size_of::<&i64>()); //8
        println!("i32 size = {}", mem::size_of::<i32>());   //4
        println!("i64 size = {}", mem::size_of::<i64>());   //8
        println!("A size = {}", mem::size_of::<A>());   //8 + 16 = 24，i32占4bytes对齐为8bytes，切片占16bytes
    }

    ///## 动态尺寸类型 - trait object
    #[test]
    fn trait_objects_layout() {
        trait MyTrait {
            fn test();
        }
        // 下面两行会报错
        // println!("MyTrait size = {}", mem::size_of::<dyn MyTrait>());
        // println!("&MyTrait size = {}", mem::size_of::<&dyn MyTrait>());
    }

    ///## 零大小类型 (ZSTs)
    #[test]
    fn zst() {
        struct Nothing; // 无字段意味着没有大小

        // 所有字段都无大小意味着整个结构体无大小
        struct LotsOfNothing {
            foo: Nothing,
            qux: (),
            // 空元组无大小
            baz: [u8; 0], // 空数组无大小
        }

        println!("Nothing size = {}", mem::size_of::<Nothing>());
        println!("LotsOfNothing size = {}", mem::size_of::<LotsOfNothing>());
    }
}

#[cfg(test)]
mod hrtb {
    struct Closure<F> {
        data: (u8, u16),
        func: F,
    }

    impl<F> Closure<F>
        where for<'a> F: Fn(&'a (u8, u16)) -> &'a u8, //在Fn trait 之外，我们遇到 HRTB 的地方不多，即使是那些地方，我们也有这个很好的魔法糖来处理普通的情况。
    {
        fn call(&self) -> &u8 {
            (self.func)(&self.data)
        }
    }

    fn do_it(data: &(u8, u16)) -> &u8 { &data.0 }

    #[test]
    fn main() {
        let clo = Closure { data: (0, 1), func: do_it };
        println!("{}", clo.call());
    }
}

#[cfg(test)]
mod dot {
    use std::rc::Rc;

    ///# 点运算符的魔法
    /// - 方法调用的点操作符看起来简单，实际上非常不简单，它在调用时，会发生很多魔法般的类型转换，例如：自动引用、自动解引用，强制类型转换直到类型能匹配等。
    ///
    /// - 假设有一个方法 foo，它有一个接收器(接收器就是 self、&self、&mut self 参数)。如果调用 value.foo()，编译器在调用 foo 之前，需要决定到底使用哪个 Self 类型来调用。现在假设 value 拥有类型 T。
    ///
    /// - 再进一步，我们使用完全限定语法来进行准确的函数调用:
    ///
    ///     1. 首先，编译器检查它是否可以直接调用 T::foo(value)，称之为值方法调用
    ///     2. 如果上一步调用无法完成(例如方法类型错误或者特征没有针对 Self 进行实现，上文提到过特征不能进行强制转换)，那么编译器会尝试增加自动引用，例如会尝试以下调用： <&T>::foo(value) 和 <&mut T>::foo(value)，称之为引用方法调用
    ///     3. 若上面两个方法依然不工作，编译器会试着解引用 T ，然后再进行尝试。这里使用了 Deref 特征 —— 若 T: Deref<Target = U> (T 可以被解引用为 U)，那么编译器会使用 U 类型进行尝试，称之为解引用方法调用
    ///     4. 若 T 不能被解引用，且 T 是一个定长类型(在编译器类型长度是已知的)，那么编译器也会尝试将 T 从定长类型转为不定长类型，例如将 [i32; 2] 转为 [i32]
    ///     5. 若还是不行，那...没有那了，最后编译器大喊一声：汝欺我甚，不干了！
    #[test]
    fn main() {
        let array = Rc::new(Box::new([1, 2, 3]));
        let first_entry = array[0];
        println!("{}",first_entry);
    }
}

#[cfg(test)]
mod _drop {
    ///## 赋值的时候（绑定变量），如果之前被初始化过，那么需要先析构，再覆盖;如果没有被初始化过，那么直接覆盖
    #[derive(Debug)]
    struct Name {
        name: &'static str,
    }

    impl Drop for Name {
        fn drop(&mut self) {
            println!("Dropping {}", self.name);
        }
    }

    #[test]
    fn main() {
        {
            let a = Name { name: "aa" }; //a未初始化，直接覆盖
            println!("1----------------");
            let b = a;              //b未被初始化，直接覆盖

            println!("2----------------");
            let mut c = Name { name: "cc" };   //c未被初始化，直接覆盖

            println!("3----------------");
            c = b;                 //c之前已经被初始化了，需要先回收，然后再用b绑定的值去覆盖
            //此处应该调用cc的drop函数,c绑定的就是之前aa
            println!("c == {:?}", c);
            println!("4----------------");
            //调用aa的析构函数
        }
        println!("At end of main");
    }
}