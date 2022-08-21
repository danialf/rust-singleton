#![allow(dead_code)]

mod step1 {
    pub static mut SINGLETON: Singleton = Singleton::new(42);
    pub struct Singleton {
        inner: u32,
    }

    impl Singleton {
        pub const fn new(value: u32) -> Self {
            Singleton { inner: value }
        }

        pub fn instance(&mut self) -> &mut u32 {
            let instance = &mut self.inner;
            instance
        }
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn singleton() {
            let instance1 = unsafe { super::SINGLETON.instance() };
            let instance2 = unsafe { &mut super::SINGLETON };

            assert_eq!(*instance1, instance2.inner);
        }
    }
}

mod step2 {
    pub static mut SINGLETON: Singleton = Singleton::new();
    pub struct Singleton {
        inner: Option<u32>,
    }

    impl Singleton {
        pub const fn new() -> Self {
            Singleton { inner: None }
        }

        fn init(&mut self, value: u32) {
            self.inner = Some(value);
        }

        pub fn instance(&mut self) -> &mut Option<u32> {
            let instance = &mut self.inner;
            instance
        }
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn singleton() {
            let instance1 = unsafe { super::SINGLETON.instance() };
            let instance2 = unsafe { &mut super::SINGLETON };
            *instance1 = Some(42);

            assert_eq!(*instance1, instance2.inner);
        }
    }
}

mod step3 {
    use std::sync::Once;
    pub static mut SINGLETON: Singleton = Singleton::new(|| 0);
    pub static ONCE: Once = Once::new();
    pub struct Singleton {
        inner: Option<u32>,
        init: fn() -> u32,
    }

    impl Singleton {
        pub const fn new(init: fn() -> u32) -> Self {
            Singleton { inner: None, init }
        }

        pub fn instance(&mut self) -> &mut u32 {
            ONCE.call_once(|| {
                let init = self.init;
                let value = init();

                unsafe {
                    SINGLETON.inner = Some(value);
                }
                println!("Initializing singleton value");
            });
            println!("accessing singleton value");
            let instance = &mut self.inner;
            instance.as_mut().unwrap()
        }
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn singleton() {
            let instance1 = unsafe { super::SINGLETON.instance() };
            let instance2 = unsafe { super::SINGLETON.instance() };

            assert_eq!(*instance1, *instance2);

            *instance1 = 42;

            assert_eq!(*instance1, *instance2);
        }
    }
}

mod step4 {
    use std::mem::MaybeUninit;
    use std::sync::Once;

    pub static mut SINGLETON: MaybeUninit<Singleton> = MaybeUninit::uninit();
    pub static ONCE: Once = Once::new();
    pub static mut INIT: Option<fn() -> u32> = None;

    pub struct Singleton {
        inner: u32,
    }

    pub fn instance() -> &'static mut u32 {
        ONCE.call_once(|| {
            unsafe {
                if let None = INIT {
                    panic!("Singeleton must be initialized before its being used.")
                }
            }
            let init = unsafe { INIT.unwrap() };
            let value = init();

            let singleton = Singleton { inner: value };

            unsafe {
                SINGLETON.write(singleton);
            }
        });
        unsafe { &mut SINGLETON.assume_init_mut().inner }
    }

    #[cfg(test)]
    mod tests {

        #[test]
        fn singleton() {
            unsafe {
                super::INIT = Some(|| 42);
            }
            let instance1 = super::instance();
            let instance2 = super::instance();

            assert_eq!(*instance1, *instance2);

            *instance1 = 24;

            assert_eq!(*instance1, *instance2);
        }
    }
}

mod step5 {
    use std::mem::MaybeUninit;
    use std::sync::Mutex;
    use std::sync::Once;

    pub static mut SINGLETON: MaybeUninit<Singleton> = MaybeUninit::uninit();
    pub static ONCE: Once = Once::new();
    pub static mut INIT: Option<fn() -> u32> = None;

    pub struct Singleton {
        inner: Mutex<u32>,
    }

    pub fn instance() -> &'static mut Singleton {
        ONCE.call_once(|| {
            unsafe {
                if let None = INIT {
                    panic!("Singeleton must be initialized before its being used.")
                }
            }
            let init = unsafe { INIT.unwrap() };
            let value = init();

            let singleton = Singleton {
                inner: Mutex::new(value),
            };

            unsafe {
                SINGLETON.write(singleton);
            }
        });
        unsafe { SINGLETON.assume_init_mut() }
    }

    #[cfg(test)]
    mod tests {

        #[test]
        fn singleton() {
            unsafe {
                super::INIT = Some(|| 42);
            }
            let instance1 = super::instance();
            let instance2 = super::instance();

            assert_eq!(*instance1.inner.lock().unwrap(), 42);
            assert_eq!(*instance2.inner.lock().unwrap(), 42);

            *instance1.inner.get_mut().unwrap() = 24;

            assert_eq!(*instance2.inner.lock().unwrap(), 24);

            *instance2.inner.get_mut().unwrap() = 142;

            assert_eq!(*instance1.inner.lock().unwrap(), 142);
        }
    }
}

mod step6 {

    macro_rules! Singleton {
        ($t: ty,$name: ident, $e: expr) => {
            pub mod  $name{
                use std::mem::MaybeUninit;
                use std::sync::Mutex;
                use std::sync::Once;

                static mut SINGLETON: MaybeUninit<Singleton> = MaybeUninit::uninit();
                static ONCE: Once = Once::new();
                static mut INIT: fn() -> $t = $e;

                pub struct Singleton {
                    inner: Mutex<$t>,
                }

                impl Singleton{
                    pub fn inner(&mut self) -> &mut Mutex<$t>
                    {
                        &mut self.inner
                    }
                }

                pub fn instance() -> &'static mut Singleton {
                    ONCE.call_once(|| {
                        let init = unsafe { INIT };
                        let value = init();

                        let singleton = Singleton {
                            inner: Mutex::new(value),
                        };

                        unsafe {
                            SINGLETON.write(singleton);
                        }
                    });
                    unsafe { SINGLETON.assume_init_mut() }
                }
        }
        };
    }

    #[cfg(test)]
    mod tests {

        Singleton!(u32, singletonu32, || 42);
        Singleton!(u64, singletonu64, || 42);
        Singleton!(f64, singletonf64, || 42.);

        #[test]
        fn singleton() {
            let instance1 = singletonu32::instance();
            let instance2 = singletonu32::instance();

            assert_eq!(*instance1.inner().lock().unwrap(), 42);
            assert_eq!(*instance2.inner().lock().unwrap(), 42);

            *instance1.inner().get_mut().unwrap() = 24;

            assert_eq!(*instance2.inner().lock().unwrap(), 24);

            *instance2.inner().get_mut().unwrap() = 142;

            assert_eq!(*instance1.inner().lock().unwrap(), 142);
        }

        #[test]
        fn singleton_multi_instance() {
            let instance1 = singletonu64::instance();
            let instance2 = singletonu64::instance();
            let instance3 = singletonf64::instance();

            assert_eq!(*instance1.inner().lock().unwrap(), 42);
            assert_eq!(*instance2.inner().lock().unwrap(), 42);
            assert_eq!(*instance3.inner().lock().unwrap(), 42.);
            
            *instance3.inner().get_mut().unwrap() = 24.;
            *instance1.inner().get_mut().unwrap() = 24;

            assert_eq!(*instance2.inner().lock().unwrap(), 24);

            *instance2.inner().get_mut().unwrap() = 142;

            assert_eq!(*instance1.inner().lock().unwrap(), 142);
            drop(instance1);
            assert_eq!(*instance2.inner().lock().unwrap(), 142);
            assert_eq!(*instance3.inner().lock().unwrap(), 24.);
        }
    }
}
