use lua51_vm::{bytecode, vm::VirtualMachine};
use tokio::{fs::File, io::BufReader};

#[tokio::main]
async fn main() {
    let file = File::open("luac.out").await.unwrap();
    let mut reader = BufReader::new(file);
    let f = bytecode::read_bytecode(&mut reader).await;

    let mut vm = VirtualMachine::new();
    vm.load_std_libraries();
    let r = vm.execute(f.unwrap(), None, None);
    dbg!(&r);
}
