use crate::context::WindowContext;

/// 一个标记trait，表示可以存储于窗口状态中的类型
pub trait Global: 'static {
    // This trait is intentionally left empty, by virtue of being a marker trait.
    //
    // Use additional traits with blanket implementations to attach functionality
    // to types that implement `Global`.
}

pub trait ReadGlobal {
    /// 返回实现类型的全局实例。
    ///
    /// 如果该类型的全局没有分配，就会陷入恐慌。
    fn global(cx: &WindowContext) -> &Self;
}

impl<T: Global> ReadGlobal for T {
    fn global(cx: &WindowContext) -> &Self {
        cx.global::<T>()
    }
}

/// A trait for updating a global value in the context.
pub trait UpdateGlobal {
    /// Set the global instance of the implementing type.
    fn set_global<C>(cx: &mut WindowContext, global: Self);
}

impl<T: Global> UpdateGlobal for T {
    fn set_global<C>(cx: &mut WindowContext, global: Self) {
        cx.set_global(global)
    }
}
