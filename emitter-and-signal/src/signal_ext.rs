use ext_trait::extension;

use super::signal::Signal;

#[extension(pub trait SignalExt)]
impl<T> Signal<T> {}
