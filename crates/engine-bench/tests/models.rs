use engine_api::{run, CancelToken, Status, MAX_MEMBERS, MAX_NODES};
use engine_bench::{
    concrete_young_pa, corner_t0, corner_tc, corner_tl, file_sha256, izz_about_master, mesh_counts,
    official_models, rectangle_section, repository_index, roof_node, shear_modulus_pa,
    spectral_acceleration_m_s2, story_mass_kg, translate, AnalysisKind, Axis, BenchModel,
    BENCH_MODEL_SCHEMA_PATH, FC_MPA, GRAVITY_M_S2, INDEX_PATH, LATERAL_WEIGHT_FRACTION, PLAN_M,
    POISSON, SA_UNIT, SPAN_M,
};
use serde_json::Value;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_repo(relative: &str) -> String {
    std::fs::read_to_string(repo_root().join(relative)).unwrap_or_else(|error| {
        panic!("could not read {relative}: {error}");
    })
}

#[test]
fn official_sizes_match_the_engine_limit() {
    assert_eq!(mesh_counts(3), (64, 120));
    assert_eq!(mesh_counts(8), (144, 320));
    assert_eq!(mesh_counts(15), (256, 600));
    let (nodes, members) = mesh_counts(16);
    assert!(nodes > MAX_NODES);
    assert!(members < MAX_MEMBERS);
    assert_eq!(MAX_NODES, 256);
    assert_eq!(MAX_MEMBERS, 1024);
}

#[test]
fn official_models_are_deterministic_and_explicit() {
    let models = official_models();
    assert_eq!(models, official_models());
    let expected = [("rc3d-3s", 3), ("rc3d-8s", 8), ("rc3d-15s", 15)];
    for (model, (id, stories)) in models.iter().zip(expected) {
        assert_eq!(model.id, id);
        assert_eq!(model.schema, "bench-model/1");
        assert_eq!(model.units, "SI");
        assert_eq!(model.stories.len(), stories);
        assert_eq!(model.nodes.len(), mesh_counts(stories).0);
        assert_eq!(model.members.len(), mesh_counts(stories).1);
        assert!(model.nodes.len() <= MAX_NODES);
        assert!(model.members.len() <= MAX_MEMBERS);
        assert_eq!(model.modal.n, 12);
        assert_eq!(model.combination.method, "cqc");
        assert_eq!(model.combination.zeta, model.spectrum.zeta);
        let direction = model.spectrum.direction.as_ref().unwrap();
        assert_eq!(direction.axis, Axis::X);
        assert_eq!(direction.vector, [1.0, 0.0, 0.0]);
        assert_eq!(model.spectrum.sa.unit, SA_UNIT);
        assert_ne!(model.spectrum.sa.unit, "g");
        for member in &model.members {
            let orientation = member.orientation.as_ref().unwrap();
            assert_eq!(orientation.kind, "local_z_vector");
            assert_eq!(orientation.vector.len(), 3);
        }
        for story in &model.stories {
            assert!(story.mass.m > 0.0);
            assert!(story.mass.izz > 0.0);
        }
    }
}

#[test]
fn mass_inertia_and_spectrum_follow_the_published_formulas() {
    let mass = story_mass_kg();
    let master = [SPAN_M, SPAN_M];
    let centroid = [PLAN_M / 2.0, PLAN_M / 2.0];
    let izz = izz_about_master(mass, master, centroid);
    let young = concrete_young_pa(FC_MPA);
    let shear = shear_modulus_pa(young, POISSON);
    let (area, iy, iz, torsion) = rectangle_section(0.70, 0.70);
    for model in official_models() {
        assert_eq!(model.stories[0].mass.m, mass);
        assert_eq!(model.stories[0].mass.izz, izz);
        let column = &model.sections[0];
        assert_eq!(column.young, young);
        assert_eq!(column.shear, Some(shear));
        assert_eq!(column.area, area);
        assert_eq!(column.iy, Some(iy));
        assert_eq!(column.iz, Some(iz));
        assert_eq!(column.j, Some(torsion));
        let lateral = &model.load_patterns.lateral[0].values[0];
        assert_eq!(lateral.dof, engine_bench::Dof::Ux);
        assert_eq!(lateral.value, LATERAL_WEIGHT_FRACTION * mass * GRAVITY_M_S2);
        for pair in &model.spectrum.sa.pairs {
            assert_eq!(pair[1], spectral_acceleration_m_s2(pair[0]));
        }
    }
    let plateau = spectral_acceleration_m_s2((corner_t0() + corner_tc()) / 2.0);
    let in_g = plateau / GRAVITY_M_S2;
    assert!(plateau > 0.5, "Sa must be in m/s^2, not in g");
    assert!(in_g < 0.5);
    let tl = corner_tl();
    assert!((spectral_acceleration_m_s2(tl) - spectral_acceleration_m_s2(tl)).abs() < 1e-12);
}

#[test]
fn translator_rejects_the_engine_limits_and_a_g_spectrum() {
    let mut oversized = official_models()[0].clone();
    while oversized.nodes.len() <= MAX_NODES {
        let mut node = oversized.nodes[0].clone();
        node.id = 100_000 + oversized.nodes.len() as u32;
        oversized.nodes.push(node);
    }
    let nodes_error = translate(&oversized, AnalysisKind::Static).unwrap_err();
    assert!(nodes_error.to_string().contains("MAX_NODES"));

    let mut too_many_members = official_models()[0].clone();
    while too_many_members.members.len() <= MAX_MEMBERS {
        let mut member = too_many_members.members[0].clone();
        member.id = 100_000 + too_many_members.members.len() as u32;
        too_many_members.members.push(member);
    }
    let members_error = translate(&too_many_members, AnalysisKind::Static).unwrap_err();
    assert!(members_error.to_string().contains("MAX_MEMBERS"));

    let mut in_g = official_models()[0].clone();
    in_g.spectrum.sa.unit = "g".to_owned();
    assert!(translate(&in_g, AnalysisKind::Spectrum)
        .unwrap_err()
        .to_string()
        .contains("m/s^2"));
}

#[test]
fn committed_files_match_the_index_hashes() {
    let index = repository_index().unwrap();
    let text = read_repo(INDEX_PATH);
    assert_eq!(text, serde_json::to_string_pretty(&index).unwrap() + "\n");
    let document: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(document["schema"], "bench-index/1");
    assert_eq!(document["limits"]["max_nodes"], 256);
    assert_eq!(document["limits"]["max_members"], 1024);
    assert_eq!(document["bench_model_schema"], BENCH_MODEL_SCHEMA_PATH);
    for entry in &index.models {
        assert!(!entry.path.contains('\\'));
        assert_eq!(file_sha256(read_repo(&entry.path).as_bytes()), entry.sha256);
        for file in [
            &entry.engine_input.linear_static,
            &entry.engine_input.modal,
            &entry.engine_input.spectrum,
        ] {
            assert!(!file.path.contains('\\'));
            assert_eq!(file_sha256(read_repo(&file.path).as_bytes()), file.sha256);
            let engine: Value = serde_json::from_str(&read_repo(&file.path)).unwrap();
            assert_eq!(engine["schema"], "engine-input/1");
            assert!(engine["model"]["nodes"].as_array().unwrap().len() <= MAX_NODES);
            assert!(engine["model"]["members"].as_array().unwrap().len() <= MAX_MEMBERS);
        }
        let model: BenchModel = serde_json::from_str(&read_repo(&entry.path)).unwrap();
        assert_eq!(roof_node(&model).unwrap(), entry.roof_node);
        assert_eq!(model.nodes.len(), entry.nodes);
        assert_eq!(model.members.len(), entry.members);
    }
    let counts: Vec<_> = index
        .models
        .iter()
        .map(|model| (model.id.as_str(), model.nodes, model.members))
        .collect();
    assert_eq!(
        counts,
        vec![
            ("rc3d-3s", 64, 120),
            ("rc3d-8s", 144, 320),
            ("rc3d-15s", 256, 600)
        ]
    );
}

#[test]
fn rc3d_hashes_stay_at_the_published_values() {
    let index = repository_index().unwrap();
    let expected = [
        (
            "rc3d-3s",
            "199cfb84d248877d70caa6e1df641582386a0dabfe725d8ec659007d4552d7b3",
            "4e7179735d6f1242c0140fc81495027d485e2bbce7f210d14f1b2e974c2e2ce8",
            "98bd1f9d30a820fc7de0f751941db759dc1f5c6133eb76b320aaedca1942832d",
            "05f56505679c567295fadc378305fe65fe2a92ad0f36a75057d275186e8b41f3",
        ),
        (
            "rc3d-8s",
            "33fa7da8e4af10edf89adf4d19737eab72391618c297a5952cfb059cf5efe960",
            "f4362e79562f717bbdf34b34c81f9a862fee043b8ebabed54cd82f346e5f7736",
            "d38ef438e544830c8511034471b098f016fac5b3d71ce98448119710d4184549",
            "a9198cb8bf38c12cdb1eccbb678ddf5fc0f43940ee50b247b10d36c7ea34570e",
        ),
        (
            "rc3d-15s",
            "b914e418b36a9ce8085ad31f7caecf1d512946215577ea961ccb0071e998a5b9",
            "5238f0896e222c2d1ffce4fd3c3c61ece07d763a6c17fa45659c7f6cb3d69cce",
            "0efec83b0a508f9d9ffd816c0288c29c047e1100440464372be7200f86e76e3a",
            "56fbb7f3cdc8c2431b5dcaf5053e00e451ccdc6cdf6f65c33c59a0c081b2e2ad",
        ),
    ];
    for (id, model_sha, static_sha, modal_sha, spectrum_sha) in expected {
        let entry = index.models.iter().find(|item| item.id == id).unwrap();
        assert_eq!(entry.sha256, model_sha);
        assert_eq!(entry.engine_input.linear_static.sha256, static_sha);
        assert_eq!(entry.engine_input.modal.sha256, modal_sha);
        assert_eq!(entry.engine_input.spectrum.sha256, spectrum_sha);
    }
}

#[test]
fn published_schema_defines_vector_mass_and_si_spectrum() {
    let schema: Value = serde_json::from_str(&read_repo(BENCH_MODEL_SCHEMA_PATH)).unwrap();
    assert_eq!(schema["title"], "bench-model/1");
    let required = schema["required"].as_array().unwrap();
    for key in [
        "schema",
        "id",
        "units",
        "stories",
        "grid",
        "nodes",
        "members",
        "sections",
        "diaphragms",
        "restraints",
        "load_patterns",
        "modal",
        "spectrum",
        "combination",
    ] {
        assert!(required.iter().any(|item| item == key), "{key}");
    }
    assert_eq!(
        schema["$defs"]["orientation"]["properties"]["kind"]["const"],
        "local_z_vector"
    );
    assert_eq!(
        schema["$defs"]["storyMass"]["required"],
        serde_json::json!(["m", "Izz"])
    );
    assert_eq!(
        schema["$defs"]["saTable"]["properties"]["unit"]["const"],
        "m/s^2"
    );
    let description = schema["$defs"]["saTable"]["description"].as_str().unwrap();
    assert!(description.contains("m/s^2"));
    assert!(description.contains("No está en g"));
    assert_eq!(schema["properties"]["nodes"]["maxItems"], 256);
    assert_eq!(schema["properties"]["members"]["maxItems"], 1024);
    assert_eq!(schema["x-mamut-orientation"]["not"], "angle");
    assert_eq!(schema["x-mamut-sa"]["not"], "g");
}

#[test]
fn static_pointer_names_the_lateral_case() {
    let index = repository_index().unwrap();
    for model in official_models() {
        let entry = index
            .models
            .iter()
            .find(|item| item.id == model.id)
            .unwrap();
        assert_eq!(
            entry.pointers.static_roof_displacement.case.as_deref(),
            Some("lateral"),
            "{}",
            model.id
        );
        assert!(
            entry
                .pointers
                .static_roof_displacement
                .json_pointer
                .starts_with("/cases/1/"),
            "{}",
            model.id
        );
        let document: Value =
            serde_json::from_str(&translate(&model, AnalysisKind::Static).unwrap()).unwrap();
        let cases = document["cases"].as_array().unwrap();
        assert!(cases.len() >= 2, "{}", model.id);
        assert_eq!(cases[0]["name"], "gravity", "{}", model.id);
        assert_eq!(
            cases[1]["name"].as_str(),
            entry.pointers.static_roof_displacement.case.as_deref(),
            "{}",
            model.id
        );
    }
}

#[test]
fn three_story_results_follow_the_index_pointers() {
    let model = official_models()
        .into_iter()
        .find(|item| item.id == "rc3d-3s")
        .unwrap();
    let entry = repository_index()
        .unwrap()
        .models
        .into_iter()
        .find(|item| item.id == "rc3d-3s")
        .unwrap();
    assert_eq!(entry.roof_node, 54);
    assert_eq!(
        entry.pointers.static_roof_displacement.json_pointer,
        "/cases/1/nodes/53/displacements/0"
    );
    assert_eq!(
        entry.pointers.cqc_roof_displacement.json_pointer,
        "/envelopes/1/nodes/53/displacements/0"
    );

    let static_result = solve(&translate(&model, AnalysisKind::Static).unwrap());
    assert_eq!(
        static_result["cases"][1]["case"], "lateral",
        "the static pointer points to the second case"
    );
    assert_eq!(
        walk(&static_result, "/cases/1/nodes/53/id"),
        &serde_json::json!(entry.roof_node)
    );
    let static_ux = walk(
        &static_result,
        &entry.pointers.static_roof_displacement.json_pointer,
    )
    .as_f64()
    .unwrap();
    assert!(static_ux.is_finite() && static_ux > 0.0);

    let modal_result = solve(&translate(&model, AnalysisKind::Modal).unwrap());
    let period = walk(&modal_result, &entry.pointers.period.json_pointer)
        .as_f64()
        .unwrap();
    assert!(period.is_finite() && period > 0.0);

    let spectrum_result = solve(&translate(&model, AnalysisKind::Spectrum).unwrap());
    assert!(spectrum_result["envelopes"][1]["direction"].is_null());
    assert_eq!(
        walk(&spectrum_result, "/envelopes/1/nodes/53/id"),
        &serde_json::json!(entry.roof_node)
    );
    let cqc_ux = walk(
        &spectrum_result,
        &entry.pointers.cqc_roof_displacement.json_pointer,
    )
    .as_f64()
    .unwrap();
    assert!(cqc_ux.is_finite() && cqc_ux >= 0.0);
}

fn solve(input: &str) -> Value {
    let outcome = run(input, &CancelToken::new(), |_| {});
    assert!(
        matches!(outcome.status, Status::Ok | Status::Warning),
        "{}",
        outcome.document
    );
    serde_json::from_str(&outcome.document).unwrap()
}

fn walk<'a>(value: &'a Value, pointer: &str) -> &'a Value {
    let mut current = value;
    for part in pointer.trim_start_matches('/').split('/') {
        current = if let Ok(index) = part.parse::<usize>() {
            &current[index]
        } else {
            &current[part]
        };
    }
    current
}

#[test]
fn orchestrator_logic_passes_its_python_tests() {
    let status = std::process::Command::new("python3")
        .arg("scripts/eng-val-006/test_orchestrator.py")
        .current_dir(repo_root())
        .status()
        .expect("python3");
    assert!(status.success());
}
