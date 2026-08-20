mod experiments;
mod fabricate;
mod tdh_helpers;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("============================================================");
    println!("  TDH Fabricate Experiment");
    println!("  Understanding TdhGetEventInformation field requirements");
    println!("============================================================");
    println!();
    println!("NOTE: Experiments 1, 3, 5, 6, 7 require administrator privileges");
    println!("      for kernel ETW tracing. They will be skipped if not admin.");
    println!("      Experiments 2 and 4 work without admin (pure fabrication).");
    println!();

    // These experiments work WITHOUT admin (pure fabrication, no ETW session)
    experiments::experiment_2_minimal_fabrication();
    experiments::experiment_4_version_probing();

    // These experiments need kernel tracing (admin required)
    // They gracefully skip if tracing fails
    experiments::experiment_1_baseline();
    experiments::experiment_3_field_sensitivity();
    experiments::experiment_5_modify_real_record();
    experiments::experiment_6_flags_and_properties();
    experiments::experiment_7_userdata_effects();

    println!();
    println!("============================================================");
    println!("  All experiments complete.");
    println!("============================================================");
}
