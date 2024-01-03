use regex::Regex;
use std::{
    io::Result,
    process::{Command, ExitStatus, Stdio},
    str::from_utf8,
};

const SATA_CONTROLLER_NAME: &str = "SATA Controller";
const IDE_CONTROLLER_NAME: &str = "IDE Controller";
const SATA_CONTROLLER: &str = "IntelAhci";
const IDE_CONTROLLER: &str = "PIIX4";

fn main() {
    let mut vbox = VBox::new();
    let mut res = vbox.showvminfo(true).expect("Failed to check the VM");
    if res.code().unwrap() != 0 {
        res = vbox.createvm(true).expect("Failed to create a new VM");
    }

    res = vbox
        .enable_ioapic()
        .expect("Failed to set ioapic for the VM");
    res = vbox
        // FIXME get memory
        .set_memory(4048, 128)
        .expect("Failed to set memory for the VM");
    res = vbox.set_nic1().expect("Failed to set nic1 for the VM");
    let mut medium = vbox.showmediuminfo().expect("Failed to get medium info");
    if let None = medium {
        res = vbox
            // TODO get size
            .createhd(20000)
            .expect("Failed to set medium for the VM");
        medium = vbox.showmediuminfo().expect("Failed to get medium info 2");
    }
    let medium = medium.unwrap();
    vbox.set_medium_id(medium.as_str());

    let sata_controller = vbox
        .get_sata_controller()
        .expect("Failed to get sata controller");
    if sata_controller.code().unwrap() != 0 {
        res = vbox
            .set_sata_controller()
            .expect("Failed to set sata controller");
        res = vbox
            .attach_sata_controller()
            .expect("Failed to attach sata controller");
    }

    let ide_controller = vbox
        .get_ide_controller()
        .expect("Failed to get ide controller");
    if ide_controller.code().unwrap() != 0 {
        res = vbox
            .set_ide_controller()
            .expect("Failed to set ide controleer");
        res = vbox
            .attach_ide_controller()
            .expect("Failed to attach ide controller");
    }
    res = vbox.modifyvm().expect("Failed to modify VM");

    res = vbox.install_os().expect("Failed to install OS to VM");
    res = vbox.postinstall().expect("Failed to postinstall OS to VM");
}

struct VBox<'a> {
    exe: &'a str,
    name: &'a str,
    ostype: &'a str,
    basefolder: &'a str,
    hdd_name: &'a str,
    modifyvm: &'a str,
    medium_id: &'a str,
    iso: &'a str,
    pass_file: &'a str,
}

impl<'a> VBox<'a> {
    fn new() -> Self {
        VBox {
            exe: "VBoxManage",
            name: "Ubuntu",
            ostype: "Ubuntu_64",
            basefolder: "C:/VMs",
            hdd_name: "Ubuntu_DISK.vdi",
            modifyvm: "modifyvm",
            medium_id: "MEDIUM_ID_DEFAULT",
            // TODO set iso
            iso: "C:/Users/User/Downloads/ubuntu-22.04.3-live-server-amd64.iso",
            // TODO provide pass
            pass_file: "C:/Users/User/Projects/vbox/tmp/pass",
        }
    }

    fn showvminfo(&self, quiet: bool) -> Result<ExitStatus> {
        Command::new(self.exe)
            .arg("showvminfo")
            .arg(self.name)
            .stdout(match quiet {
                true => Stdio::null(),
                false => Stdio::inherit(),
            })
            .status()
    }

    fn createvm(&self, quiet: bool) -> Result<ExitStatus> {
        Command::new(self.exe)
            .arg("createvm")
            .arg("--name")
            .arg(self.name)
            .arg("--ostype")
            .arg(self.ostype)
            .arg("--register")
            .arg("--basefolder")
            .arg(self.basefolder)
            .stdout(match quiet {
                true => Stdio::null(),
                false => Stdio::inherit(),
            })
            .status()
    }

    fn enable_ioapic(&self) -> Result<ExitStatus> {
        Command::new(self.exe)
            .arg(self.modifyvm)
            .arg(self.name)
            .arg("--ioapic")
            .arg("on")
            .status()
    }

    fn set_memory(&self, memory: u32, vram: u8) -> Result<ExitStatus> {
        Command::new(self.exe)
            .arg(self.modifyvm)
            .arg(self.name)
            .arg("--memory")
            .arg(memory.to_string())
            .arg("--vram")
            .arg(vram.to_string())
            .status()
    }

    fn set_nic1(&self) -> Result<ExitStatus> {
        Command::new(self.exe)
            .arg(self.modifyvm)
            .arg(self.name)
            .arg("--nic1")
            .arg("nat")
            .status()
    }

    fn get_medium_name(&self) -> String {
        format!("{}/{}/{}", self.basefolder, self.ostype, self.hdd_name)
    }

    fn createhd(&self, size: u32) -> Result<ExitStatus> {
        Command::new(self.exe)
            .arg("createhd")
            .arg("--filename")
            .arg(self.get_medium_name())
            .arg("--size")
            .arg(size.to_string())
            .arg("--format")
            .arg("VDI")
            .status()
    }

    fn set_medium_id(&mut self, medium_id: &'a str) {
        self.medium_id = medium_id;
    }

    fn showmediuminfo(&self) -> Result<Option<String>> {
        let output = Command::new(self.exe)
            .arg("showmediuminfo")
            .arg(self.get_medium_name())
            .output()?;
        let re = Regex::new(r"^UUID:\s+.+\s").unwrap();
        let capts = re.captures(from_utf8(&output.stdout).unwrap());
        if let None = capts {
            return Ok(None);
        }
        let capts = capts.unwrap();
        let res = capts.get(0).unwrap().as_str().to_string();
        let res = Regex::new(r"[(UUID:)\s+]+")
            .unwrap()
            .replace_all(&res, "")
            .to_string();
        Ok(Some(res))
    }

    fn encryptmedium(&self, size: u32) -> Result<ExitStatus> {
        Command::new(self.exe)
            .arg("encryptmedium")
            .arg(self.medium_id)
            .arg("--newpassword")
            .arg(self.pass_file)
            .arg(size.to_string())
            .arg("--newpassword-id")
            // TODO provide id
            .arg("1")
            .arg("--cipher")
            .arg("AES-XTS128-PLAIN64")
            .status()
    }

    fn get_sata_controller(&self) -> Result<ExitStatus> {
        Command::new(self.exe)
            .arg("storagectl")
            .arg(self.name)
            .arg("--name")
            .arg(SATA_CONTROLLER_NAME)
            .arg("--controller")
            .arg(SATA_CONTROLLER)
            .stdout(Stdio::null())
            .status()
    }

    fn set_sata_controller(&self) -> Result<ExitStatus> {
        Command::new(self.exe)
            .arg("storagectl")
            .arg(self.name)
            .arg("--name")
            .arg(SATA_CONTROLLER_NAME)
            .arg("--add")
            .arg("sata")
            .arg("--controller")
            .arg(SATA_CONTROLLER)
            .status()
    }

    fn get_ide_controller(&self) -> Result<ExitStatus> {
        Command::new(self.exe)
            .arg("storagectl")
            .arg(self.name)
            .arg("--name")
            .arg(IDE_CONTROLLER_NAME)
            .arg("--controller")
            .arg(IDE_CONTROLLER)
            .stdout(Stdio::null())
            .status()
    }

    fn set_ide_controller(&self) -> Result<ExitStatus> {
        Command::new(self.exe)
            .arg("storagectl")
            .arg(self.name)
            .arg("--name")
            .arg(IDE_CONTROLLER_NAME)
            .arg("--add")
            .arg("ide")
            .arg("--controller")
            .arg(IDE_CONTROLLER)
            .status()
    }

    fn attach_sata_controller(&self) -> Result<ExitStatus> {
        Command::new(self.exe)
            .arg("storageattach")
            .arg(self.name)
            .arg("--storagectl")
            .arg(SATA_CONTROLLER_NAME)
            .arg("--port")
            .arg("0")
            .arg("--device")
            .arg("0")
            .arg("--type")
            .arg("hdd")
            .arg("--medium")
            .arg(self.get_medium_name())
            .status()
    }

    fn attach_ide_controller(&self) -> Result<ExitStatus> {
        Command::new(self.exe)
            .arg("storageattach")
            .arg(self.name)
            .arg("--storagectl")
            .arg(IDE_CONTROLLER_NAME)
            .arg("--port")
            .arg("0")
            .arg("--device")
            .arg("0")
            .arg("--type")
            .arg("dvddrive")
            .arg("--medium")
            .arg(self.iso)
            .status()
    }
    fn modifyvm(&self) -> Result<ExitStatus> {
        Command::new(self.exe)
            .arg("modifyvm")
            .arg(self.name)
            .arg("--boot1")
            .arg("dvd")
            .arg("--boot2")
            .arg("disk")
            .arg("--boot3")
            .arg("none")
            .arg("--boot4")
            .arg("none")
            .arg("--cpus")
            // TODO provide cpus
            .arg("4")
            .status()
    }

    fn install_os(&self) -> Result<ExitStatus> {
        Command::new(self.exe)
            .arg("unattended")
            .arg("install")
            .arg(self.name)
            .arg("--iso")
            .arg(self.iso)
            .arg("--user")
            .arg("test")
            .arg("--full-user-name")
            .arg("Test")
            .arg("--password")
            .arg("test")
            .arg("--install-additions")
            .arg("--time-zone")
            .arg("CET")
            .status()
    }
    fn postinstall(&self) -> Result<ExitStatus> {
        Command::new(self.exe)
            .arg("modifyvm")
            .arg(self.name)
            .arg("--boot1")
            .arg("disk")
            .arg("--boot2")
            .arg("dvd")
            .arg("--boot3")
            .arg("none")
            .arg("--boot4")
            .arg("none")
            .status()
    }
}
