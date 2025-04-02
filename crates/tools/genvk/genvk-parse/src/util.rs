pub trait Extract {
    type Output;
}

impl<T: Extract> Extract for Option<T> {
    type Output = T;
}

impl<T: Extract> Extract for Vec<T> {
    type Output = Vec<T>;
}

impl Extract for bool {
    type Output = bool;
}

impl Extract for i32 {
    type Output = i32;
}

impl Extract for u32 {
    type Output = u32;
}

impl Extract for String {
    type Output = String;
}
