#[cfg(test)]
mod tests {
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
