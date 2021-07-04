use oxidebpf::{Program, ProgramBlueprint, ProgramGroup, ProgramType, ProgramVersion};

fn main() {
    let program_blueprint =
        ProgramBlueprint::new(&std::fs::read("bpf/filter_program_x86_64").unwrap(), None).unwrap();
    let mut program_group = ProgramGroup::new(
        program_blueprint,
        vec![ProgramVersion::new(vec![Program::new(
            ProgramType::Xdp,
            "ntp_filter",
            vec!["eth1"],
        )])],
        None,
    );

    program_group.load().expect("could not load program");
    let channel = program_group.get_receiver().unwrap();
    loop {
        let msg = channel.recv().unwrap();
        println!("Map name: {}; CPU: {};\nData: {:?}", msg.0, msg.1, msg.2);
    }
}
