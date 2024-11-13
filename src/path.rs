#[derive(Default, Clone)]
pub struct Path {
    path: Vec<String>,
}

impl Path {
    /// 创建一个新的 `Path` 实例
    pub fn new() -> Self {
        Default::default()
    }

    /// 获取路径的字符串表示
    pub fn path(&self) -> String {
        let sep = Self::separator();
        self.path.join(&sep)
    }

    /// 添加一个路径组件
    pub fn push<T: AsRef<str>>(&mut self, path: T) {
        self.path.push(path.as_ref().to_string());
    }

    /// 移除最后一个路径组件
    pub fn pop(&mut self) -> Option<String> {
        self.path.pop()
    }

    /// 获取路径分隔符
    pub fn separator() -> String {
        #[cfg(windows)]
        return "\\".to_string();
        #[cfg(unix)]
        return "/".to_string();
    }
}

impl<T: AsRef<str>> From<T> for Path {
    /// 从字符串创建 `Path` 实例
    fn from(s: T) -> Self {
        let sep = Self::separator();
        let path = s.as_ref().split(&sep).map(|s| s.to_string()).collect();
        Path { path }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_push() {
        // Arrange
        let mut path = Path::new();

        // Act
        path.push("HKEY_LOCAL_MACHINE");
        path.push("SOFTWARE");
        path.push("Microsoft");

        // Asset
        assert_eq!(r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft", path.path());
    }

    #[test]
    fn path_pop() {
        // Arrange
        let mut path = Path::new();

        // Act
        path.push("HKEY_LOCAL_MACHINE");
        path.push("SOFTWARE");
        path.push("Microsoft");
        path.pop();

        // Asset
        assert_eq!(r"HKEY_LOCAL_MACHINE\SOFTWARE", path.path()); // 输出:
    }

    #[test]
    fn path_from_str() {
        // Arrange
        let path_from_str = Path::from("HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft");

        // Asset
        assert_eq!(
            r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft",
            path_from_str.path()
        );
    }
}
