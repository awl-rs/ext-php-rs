//todo(eas): we need to have a fallback for php 7 and less
use std::{ffi::CString, mem::MaybeUninit};

use crate::{
    builders::FunctionBuilder,
    class::{ConstructorMeta, ConstructorResult, RegisteredClass},
    convert::IntoZval,
    error::{Error, Result},
    exception::PhpException,
    ffi::{
        zend_declare_class_constant, zend_declare_property, zend_do_implement_interface,
        zend_enum_add_case_cstr, zend_register_internal_enum,
    },
    flags::{ClassFlags, MethodFlags, PropertyFlags},
    types::{ZendClassObject, ZendObject, ZendStr, Zval},
    zend::{ClassEntry, ExecuteData, FunctionEntry},
    zend_fastcall,
};

pub struct VariantEntry {
    discriminant: String,
    value: Zval,
}

/// Builder for registering a class in PHP.
pub struct EnumBuilder {
    name: String,
    variants: Vec<VariantEntry>,
    extends: Option<&'static ClassEntry>,
    interfaces: Vec<&'static ClassEntry>,
    methods: Vec<FunctionEntry>,
    object_override: Option<unsafe extern "C" fn(class_type: *mut ClassEntry) -> *mut ZendObject>,
    constants: Vec<(String, Zval)>,
}

impl EnumBuilder {
    /// Creates a new class builder, used to build classes
    /// to be exported to PHP.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the class.
    pub fn new<T: Into<String>>(name: T) -> Self {
        Self {
            name: name.into(),
            variants: vec![],
            extends: None,
            interfaces: vec![],
            methods: vec![],
            object_override: None,
            constants: vec![],
        }
    }

    /// Sets the class builder to extend another class.
    ///
    /// # Parameters
    ///
    /// * `parent` - The parent class to extend.
    pub fn extends(mut self, parent: &'static ClassEntry) -> Self {
        self.extends = Some(parent);
        self
    }

    /// Implements an interface on the class.
    ///
    /// # Parameters
    ///
    /// * `interface` - Interface to implement on the class.
    ///
    /// # Panics
    ///
    /// Panics when the given class entry `interface` is not an interface.
    pub fn implements(mut self, interface: &'static ClassEntry) -> Self {
        assert!(
            interface.is_interface(),
            "Given class entry was not an interface."
        );
        self.interfaces.push(interface);
        self
    }

    /// Adds a method to the class.
    ///
    /// # Parameters
    ///
    /// * `func` - The function entry to add to the class.
    /// * `flags` - Flags relating to the function. See [`MethodFlags`].
    pub fn method(mut self, mut func: FunctionEntry, flags: MethodFlags) -> Self {
        func.flags = flags.bits();
        self.methods.push(func);
        self
    }

    /// Adds an enum variant to the enum.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the property to add to the class.
    /// * `value` - The value of the variant.
    ///
    /// # Panics
    ///
    /// Function will panic if the given `default` cannot be converted into a
    /// [`Zval`].
    pub fn variant(mut self, discriminant: impl Into<String>, value: impl IntoZval) -> Self {
        let discriminant = discriminant.into();
        let value = match value.into_zval(true) {
            Ok(default) => default,
            Err(_) => panic!("Invalid default value for property `{}`.", discriminant),
        };
        let variant = VariantEntry {
            discriminant,
            value,
        };

        self.variants.push(variant);
        self
    }

    /// Adds a constant to the class. The type of the constant is defined by the
    /// type of the given default.
    ///
    /// Returns a result containing the class builder if the constant was
    /// successfully added.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the constant to add to the class.
    /// * `value` - The value of the constant.
    pub fn constant<T: Into<String>>(mut self, name: T, value: impl IntoZval) -> Result<Self> {
        let value = value.into_zval(true)?;

        self.constants.push((name.into(), value));
        Ok(self)
    }

    /// Overrides the creation of the Zend object which will represent an
    /// instance of this class.
    ///
    /// # Parameters
    ///
    /// * `T` - The type which will override the Zend object. Must implement
    ///   [`RegisteredClass`]
    /// which can be derived using the [`php_class`](crate::php_class) attribute
    /// macro.
    ///
    /// # Panics
    ///
    /// Panics if the class name associated with `T` is not the same as the
    /// class name specified when creating the builder.
    pub fn object_override<T: RegisteredClass>(mut self) -> Self {
        extern "C" fn create_object<T: RegisteredClass>(_: *mut ClassEntry) -> *mut ZendObject {
            // SAFETY: After calling this function, PHP will always call the constructor
            // defined below, which assumes that the object is uninitialized.
            let obj = unsafe { ZendClassObject::<T>::new_uninit() };
            obj.into_raw().get_mut_zend_obj()
        }

        zend_fastcall! {
            extern fn constructor<T: RegisteredClass>(ex: &mut ExecuteData, _: &mut Zval) {
                let ConstructorMeta { constructor, .. } = match T::CONSTRUCTOR {
                    Some(c) => c,
                    None => {
                        PhpException::default("You cannot instantiate this class from PHP.".into())
                            .throw()
                            .expect("Failed to throw exception when constructing class");
                        return;
                    }
                };

                let this = match constructor(ex) {
                    ConstructorResult::Ok(this) => this,
                    ConstructorResult::Exception(e) => {
                        e.throw()
                            .expect("Failed to throw exception while constructing class");
                        return;
                    }
                    ConstructorResult::ArgError => return,
                };
                let this_obj = match ex.get_object::<T>() {
                    Some(obj) => obj,
                    None => {
                        PhpException::default("Failed to retrieve reference to `this` object.".into())
                            .throw()
                            .expect("Failed to throw exception while constructing class");
                        return;
                    }
                };
                this_obj.initialize(this);
            }
        }

        debug_assert_eq!(
            self.name.as_str(),
            T::CLASS_NAME,
            "Class name in builder does not match class name in `impl RegisteredClass`."
        );
        self.object_override = Some(create_object::<T>);
        self.method(
            {
                let mut func = FunctionBuilder::new("__construct", constructor::<T>);
                if let Some(ConstructorMeta { build_fn, .. }) = T::CONSTRUCTOR {
                    func = build_fn(func);
                }
                func.build().expect("Failed to build constructor function")
            },
            MethodFlags::Public,
        )
    }

    /// Builds the class, returning a reference to the class entry.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] variant if the class could not be registered.
    pub fn build(mut self) -> Result<&'static mut ClassEntry> {
        let name_const_ptr = CString::new(self.name.as_str())?.as_ptr();

        self.methods.push(FunctionEntry::end());
        let func = Box::into_raw(self.methods.into_boxed_slice()) as *const FunctionEntry;

        // todo(eas) connect with `extends`?
        let enum_type = crate::ffi::IS_UNDEF as u8;

        let enum_ = unsafe {
            zend_register_internal_enum(name_const_ptr, enum_type, func)
                .as_mut()
                .ok_or(Error::InvalidPointer)?
        };

        // disable serialization if the class has an associated object
        if self.object_override.is_some() {
            cfg_if::cfg_if! {
                if #[cfg(php81)] {
                    enum_.ce_flags |= ClassFlags::NotSerializable.bits();
                } else {
                    class.serialize = Some(crate::ffi::zend_class_serialize_deny);
                    class.unserialize = Some(crate::ffi::zend_class_unserialize_deny);
                }
            }
        }

        for iface in self.interfaces {
            unsafe {
                zend_do_implement_interface(
                    enum_,
                    iface as *const crate::ffi::_zend_class_entry
                        as *mut crate::ffi::_zend_class_entry,
                )
            };
        }

        for v in self.variants {
            let VariantEntry {
                discriminant,
                value,
            } = v;

            unsafe {
                zend_enum_add_case_cstr(
                    enum_,
                    CString::new(discriminant.as_str())?.as_ptr(),
                    value.ptr().unwrap(),
                );
            }
        }

        for (name, value) in self.constants {
            let value = Box::into_raw(Box::new(value));
            unsafe {
                zend_declare_class_constant(
                    enum_,
                    CString::new(name.as_str())?.as_ptr(),
                    name.len() as u64,
                    value,
                )
            };
        }

        if let Some(object_override) = self.object_override {
            enum_.__bindgen_anon_2.create_object = Some(object_override);
        }

        Ok(enum_)
    }
}
