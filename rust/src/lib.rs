// Copyright 2018 astonbitecode
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate jni_sys;
#[macro_use]
extern crate lazy_static;
extern crate libc;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::mem;
use std::os::raw::{c_long, c_void};
use std::sync::mpsc::Sender;

use jni_sys::{jlong, JNIEnv, jobject};

pub use self::api::Callback as Callback;
pub use self::api::ClasspathEntry as ClasspathEntry;
pub use self::api::Instance as Instance;
pub use self::api::InstanceReceiver as InstanceReceiver;
pub use self::api::InvocationArg as InvocationArg;
pub use self::api::JavaOpt as JavaOpt;
pub use self::api::Jvm as Jvm;
pub use self::api::JvmBuilder as JvmBuilder;
pub use self::api_tweaks::{get_created_java_vms, set_java_vm};

mod api;
pub(crate) mod api_tweaks;
mod utils;
mod logger;

pub mod errors;

/// Creates a new JVM, using the provided classpath entries and JVM arguments
pub fn new_jvm(classpath_entries: Vec<ClasspathEntry>, java_opts: Vec<JavaOpt>) -> errors::Result<Jvm> {
    JvmBuilder::new()
        .classpath_entries(classpath_entries)
        .java_opts(java_opts)
        .build()
}

#[no_mangle]
pub extern fn Java_org_astonbitecode_j4rs_api_invocation_NativeCallbackSupport_docallback(_jni_env: *mut JNIEnv, _class: *const c_void, ptr_address: c_long, native_invocation: jobject) {
    let pointer_from_address = ptr_address as *const ();
    let function = unsafe {
        mem::transmute::<*const (), Callback>(pointer_from_address)
    };
    let mut jvm = Jvm::new(&Vec::new(), None).expect("Could not create a j4rs Jvm while invoking deprecated callback.");
    jvm.detach_thread_on_drop(false);
    function(jvm, Instance::from(native_invocation).unwrap());
}

#[no_mangle]
pub extern fn Java_org_astonbitecode_j4rs_api_invocation_NativeCallbackToRustChannelSupport_docallbacktochannel(_jni_env: *mut JNIEnv, _class: *const c_void, ptr_address: jlong, native_invocation: jobject) {
    let mut jvm = Jvm::attach_thread().expect("Could not create a j4rs Jvm while invoking callback to channel.");
    jvm.detach_thread_on_drop(false);
    let instance_res = Instance::from(native_invocation);
    if let Ok(instance) = instance_res {
        let p = ptr_address as *mut Sender<Instance>;
        let tx = unsafe { Box::from_raw(p) };

        let result = tx.send(instance);
        mem::forget(tx);
        if let Err(error) = result {
            panic!("Could not send to the defined callback channel: {:?}", error);
        }
    }
    else {
        panic!("Could not create Instance from the NativeInvocation object...");
    }
}

#[cfg(test)]
mod lib_unit_tests {
    use std::{thread, time};
    use std::thread::JoinHandle;

    use super::{ClasspathEntry, Instance, InvocationArg, Jvm, JvmBuilder};

    #[test]
    fn create_instance_and_invoke() {
        let jvm: Jvm = JvmBuilder::new()
            .classpath_entry(ClasspathEntry::new("onemore.jar"))
            .build()
            .unwrap();

        let instantiation_args = vec![InvocationArg::from("arg from Rust")];
        let instance = jvm.create_instance("java.lang.String", instantiation_args.as_ref());
        match instance {
            Ok(i) => {
                let invocation_args = vec![InvocationArg::from(" ")];
                let invocation_result = jvm.invoke(&i, "split", &invocation_args);
                assert!(invocation_result.is_ok());
            }
            Err(error) => {
                panic!("ERROR when creating Instance: {:?}", error);
            }
        };

        let instantiation_args_2 = vec![InvocationArg::from("arg from Rust")];
        let instance_2 = jvm.create_instance("java.lang.String", instantiation_args_2.as_ref());
        match instance_2 {
            Ok(i) => {
                let invocation_args = vec![InvocationArg::from(" ")];
                let invocation_result = jvm.invoke(&i, "split", &invocation_args);
                assert!(invocation_result.is_ok());
            }
            Err(error) => {
                panic!("ERROR when creating Instance: {:?}", error);
            }
        };

        let static_invocation_result = jvm.invoke_static("java.lang.System", "currentTimeMillis", &Vec::new());
        assert!(static_invocation_result.is_ok());
    }

    //#[test]
    //#[ignore]
    fn _callback() {
        let jvm: Jvm = super::new_jvm(vec![ClasspathEntry::new("onemore.jar")], Vec::new()).unwrap();

        match jvm.create_instance("org.astonbitecode.j4rs.tests.MyTest", Vec::new().as_ref()) {
            Ok(i) => {
                let res = jvm.invoke_async(&i, "performCallback", Vec::new().as_ref(), _my_callback);
                let thousand_millis = time::Duration::from_millis(1000);
                thread::sleep(thousand_millis);
                assert!(res.is_ok());
            }
            Err(error) => {
                panic!("ERROR when creating Instance: {:?}", error);
            }
        }
    }

    #[test]
    fn callback_to_channel() {
        let jvm: Jvm = super::new_jvm(vec![ClasspathEntry::new("onemore.jar")], Vec::new()).unwrap();
        match jvm.create_instance("org.astonbitecode.j4rs.tests.MySecondTest", Vec::new().as_ref()) {
            Ok(i) => {
                let instance_receiver_res = jvm.invoke_to_channel(&i, "performCallback", Vec::new().as_ref());
                assert!(instance_receiver_res.is_ok());
                let instance_receiver = instance_receiver_res.unwrap();
                let res_chan = instance_receiver.rx().recv();
                let i = res_chan.unwrap();
                let res_to_rust = jvm.to_rust(i);
                assert!(res_to_rust.is_ok());
                let _: String = res_to_rust.unwrap();
                let millis = time::Duration::from_millis(500);
                thread::sleep(millis);
            }
            Err(error) => {
                panic!("ERROR when creating Instance: {:?}", error);
            }
        }
    }

    #[test]
    fn multiple_callbacks_to_channel() {
        let jvm: Jvm = super::new_jvm(vec![ClasspathEntry::new("onemore.jar")], Vec::new()).unwrap();
        match jvm.create_instance("org.astonbitecode.j4rs.tests.MySecondTest", Vec::new().as_ref()) {
            Ok(i) => {
                let instance_receiver_res = jvm.invoke_to_channel(&i, "performTenCallbacks", Vec::new().as_ref());
                assert!(instance_receiver_res.is_ok());
                let instance_receiver = instance_receiver_res.unwrap();
                for _i in 0..10 {
                    let thousand_millis = time::Duration::from_millis(1000);
                    let res_chan = instance_receiver.rx().recv_timeout(thousand_millis);
                    let i = res_chan.unwrap();
                    let res_to_rust = jvm.to_rust(i);
                    assert!(res_to_rust.is_ok());
                    let _: String = res_to_rust.unwrap();
                }
                let millis = time::Duration::from_millis(500);
                thread::sleep(millis);
            }
            Err(error) => {
                panic!("ERROR when creating Instance: {:?}", error);
            }
        }
    }

    #[test]
    fn multiple_callbacks_to_channel_from_multiple_threads() {
        let jvm: Jvm = super::new_jvm(vec![ClasspathEntry::new("onemore.jar")], Vec::new()).unwrap();
        match jvm.create_instance("org.astonbitecode.j4rs.tests.MySecondTest", Vec::new().as_ref()) {
            Ok(i) => {
                let instance_receiver_res = jvm.invoke_to_channel(&i, "performCallbackFromTenThreads", Vec::new().as_ref());
                assert!(instance_receiver_res.is_ok());
                let instance_receiver = instance_receiver_res.unwrap();
                for _i in 0..10 {
                    let thousand_millis = time::Duration::from_millis(1000);
                    let res_chan = instance_receiver.rx().recv_timeout(thousand_millis);
                    let i = res_chan.unwrap();
                    let res_to_rust = jvm.to_rust(i);
                    assert!(res_to_rust.is_ok());
                    let _: String = res_to_rust.unwrap();
                }
                let millis = time::Duration::from_millis(500);
                thread::sleep(millis);
            }
            Err(error) => {
                panic!("ERROR when creating Instance:  {:?}", error);
            }
        }
    }

    #[test]
    fn clone_instance() {
        let jvm: Jvm = super::new_jvm(vec![ClasspathEntry::new("onemore.jar")], Vec::new()).unwrap();
        // Create a MyTest instance
        let i_result = jvm.create_instance("org.astonbitecode.j4rs.tests.MyTest", Vec::new().as_ref());
        assert!(i_result.is_ok());
        let i_arg = i_result.unwrap();

        // Create two clones of the instance
        let i1 = jvm.clone_instance(&i_arg).unwrap();
        let i2 = jvm.clone_instance(&i_arg).unwrap();
        // Use the clones as arguments
        let invocation_res = jvm.create_instance("org.astonbitecode.j4rs.tests.MyTest", &vec![InvocationArg::from(i1)]);
        assert!(invocation_res.is_ok());
        let invocation_res = jvm.create_instance("org.astonbitecode.j4rs.tests.MyTest", &vec![InvocationArg::from(i2)]);
        assert!(invocation_res.is_ok());
    }

    //#[test]
    //#[ignore]
    fn _memory_leaks() {
        let jvm: Jvm = super::new_jvm(Vec::new(), Vec::new()).unwrap();

        for i in 0..100000000 {
            match jvm.create_instance("org.astonbitecode.j4rs.tests.MySecondTest", Vec::new().as_ref()) {
                Ok(_) => {
                    if i % 100000 == 0 {
                        println!("{}", i);
                    }
                }
                Err(error) => {
                    panic!("ERROR when creating Instance: {:?}", error);
                }
            }
        }
        let thousand_millis = time::Duration::from_millis(1000);
        thread::sleep(thousand_millis);
    }

    #[test]
    fn cast() {
        let jvm: Jvm = super::new_jvm(vec![ClasspathEntry::new("onemore.jar")], vec![]).unwrap();

        let instantiation_args = vec![InvocationArg::from("Hi")];
        let instance = jvm.create_instance("java.lang.String", instantiation_args.as_ref()).unwrap();
        jvm.cast(&instance, "java.lang.Object").unwrap();
    }

    #[test]
    fn invoke_vec() {
        let jvm: Jvm = super::new_jvm(vec![ClasspathEntry::new("onemore.jar")], vec![]).unwrap();

        match jvm.create_instance("org.astonbitecode.j4rs.tests.MyTest", Vec::new().as_ref()) {
            Ok(i) => {
                let invocation_args = vec![InvocationArg::from((vec!["arg1", "arg2", "arg3", "arg33"].as_slice(), &jvm))];
                let _ = jvm.invoke(&i, "list", &invocation_args);
            }
            Err(error) => {
                panic!("ERROR when creating Instance: {:?}", error);
            }
        }
    }

    #[test]
    fn multithread() {
        let v: Vec<JoinHandle<String>> = (0..10)
            .map(|i: i8| {
                let v = thread::spawn(move || {
                    let jvm: Jvm = super::new_jvm(Vec::new(), Vec::new()).unwrap();
                    let instantiation_args = vec![InvocationArg::from(format!("Thread{}", i))];
                    let instance = jvm.create_instance("java.lang.String", instantiation_args.as_ref()).unwrap();
                    let string: String = jvm.to_rust(instance).unwrap();
                    string
                });
                v
            })
            .collect();

        for jh in v {
            let str = jh.join();
            println!("{}", str.unwrap());
        }
    }

    #[test]
    fn use_a_java_instance_in_different_thread() {
        let jvm: Jvm = super::new_jvm(Vec::new(), Vec::new()).unwrap();
        let instantiation_args = vec![InvocationArg::from("3")];
        let instance = jvm.create_instance("java.lang.String", instantiation_args.as_ref()).unwrap();

        let jh = thread::spawn(move || {
            let jvm: Jvm = super::new_jvm(Vec::new(), Vec::new()).unwrap();
            let res = jvm.invoke(&instance, "isEmpty", &Vec::new());
            res
        });

        let join_res = jh.join();
        assert!(join_res.is_ok());
        assert!(join_res.unwrap().is_ok());
    }

    #[test]
    fn drop_and_attach_main_thread() {
        let tid = format!("{:?}", thread::current().id());
        {
            let jvm: Jvm = super::new_jvm(Vec::new(), Vec::new()).unwrap();
            let instantiation_args = vec![InvocationArg::from(tid.clone())];
            let instance = jvm.create_instance("java.lang.String", instantiation_args.as_ref()).unwrap();
            let ref tid_from_java: String = jvm.to_rust(instance).unwrap();
            assert!(&tid == tid_from_java);
        }
        {
            let jvm: Jvm = super::new_jvm(Vec::new(), Vec::new()).unwrap();
            let instantiation_args = vec![InvocationArg::from(tid.clone())];
            let instance = jvm.create_instance("java.lang.String", instantiation_args.as_ref()).unwrap();
            let ref tid_from_java: String = jvm.to_rust(instance).unwrap();
            assert!(&tid == tid_from_java);
        }
    }

    #[test]
    fn drop_and_attach_other_thread() {
        let _: Jvm = super::new_jvm(Vec::new(), Vec::new()).unwrap();
        let jh = thread::spawn(move || {
            let tid = format!("{:?}", thread::current().id());
            {
                let jvm: Jvm = super::new_jvm(Vec::new(), Vec::new()).unwrap();
                let instantiation_args = vec![InvocationArg::from(tid.clone())];
                let instance = jvm.create_instance("java.lang.String", instantiation_args.as_ref()).unwrap();
                let ref tid_from_java: String = jvm.to_rust(instance).unwrap();
                assert!(&tid == tid_from_java);
            }
            {
                let jvm: Jvm = super::new_jvm(Vec::new(), Vec::new()).unwrap();
                let instantiation_args = vec![InvocationArg::from(tid.clone())];
                let instance = jvm.create_instance("java.lang.String", instantiation_args.as_ref()).unwrap();
                let ref tid_from_java: String = jvm.to_rust(instance).unwrap();
                assert!(&tid == tid_from_java);
            }
            true
        });

        assert!(jh.join().unwrap());
    }

    fn _my_callback(jvm: Jvm, inst: Instance) {
        let string_from_java: String = jvm.to_rust(inst).unwrap();
        println!("Asynchronously got from Java: {}", string_from_java);
    }
}
