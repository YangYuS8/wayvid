// Included inside the preparation tests so these use the same authored fixture.
#[test]
fn instance_texture_precedence_decodes_only_the_final_fallback() {
    let fixture = Fixture::new();
    fixture.write("materials/image.json", br#"{"passes":[{"shader":"genericimage4","combos":{"version":2},"textures":["missing-material-default"]}]}"#);
    fixture.scene(serde_json::json!([{"image":"models/image.json","instance":{"id":7,"combos":{"version":2},"textures":["image"]}}]));
    let prepared = PreparedScene::prepare(&fixture.0, None).unwrap();
    assert_eq!(prepared.textures.len(), 1);
    assert_eq!(prepared.textures[0].rgba(), [255, 0, 0, 255]);
    fixture.scene(serde_json::json!([{"image":"models/image.json","instance":{"textures":["missing-instance"]}}]));
    assert!(PreparedScene::prepare(&fixture.0, None).is_err());
    fixture.scene(serde_json::json!([{"image":"models/image.json","instance":{"textures":["image"],"unknown":true}}]));
    assert!(PreparedScene::prepare(&fixture.0, None).is_err());
}

#[test]
fn resource_capability_shadows_instance_and_clear_restores_its_default() {
    let fixture = Fixture::new();
    fixture.write("project.json", br#"{"type":"scene","file":"scene.json","general":{"properties":{"heading":{"type":"group","value":""},"background":{"type":"scenetexture","value":""}}}}"#);
    fixture.scene(serde_json::json!([{"image":"models/image.json","instance":{"textures":["image"],"usertextures":["background"]}}]));
    let mut bytes = std::io::Cursor::new(Vec::new());
    image::RgbaImage::from_pixel(2, 3, image::Rgba([1, 2, 3, 255]))
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    let capability =
        lwe_scene::ImageSource::decode(bytes.get_ref(), scene_texture_memory_budget()).unwrap();
    let choices = BTreeMap::from([("background".into(), capability.clone())]);
    let selected =
        PreparedScene::prepare_with_resource_choices(&fixture.0, None, &BTreeMap::new(), &choices)
            .unwrap();
    assert_eq!(selected.textures.len(), 1);
    assert_eq!(
        (selected.textures[0].width, selected.textures[0].height),
        (2, 3)
    );
    assert_eq!(
        selected.textures[0].image.as_ref().unwrap().allocation_id(),
        capability.allocation_id()
    );
    assert!(selected.textures[0].rgba.is_empty());
    assert_eq!(selected.textures[0].rgba(), capability.rgba());
    let cleared = PreparedScene::prepare_with_resource_choices(
        &fixture.0,
        None,
        &BTreeMap::new(),
        &BTreeMap::new(),
    )
    .unwrap();
    assert_eq!(cleared.textures[0].rgba(), [255, 0, 0, 255]);
    assert!(
        PreparedScene::prepare_with_resource_choices(
            &fixture.0,
            None,
            &BTreeMap::new(),
            &BTreeMap::from([("heading".into(), capability.clone())])
        )
        .is_err()
    );
    assert!(
        PreparedScene::prepare_with_properties(
            &fixture.0,
            None,
            &BTreeMap::from([("background".into(), serde_json::json!("sha256:abcd"))])
        )
        .is_err()
    );
    fixture.scene(serde_json::json!([{"image":"models/image.json","instance":{"textures":["image"],"usertextures":["heading"]}}]));
    assert!(PreparedScene::prepare(&fixture.0, None).is_err());
}

#[test]
fn scoped_layer_composite_alias_rejects_other_layer_and_unbounded_names() {
    let fixture = Fixture::new();
    fixture.shake_shader();
    fixture.write(
        "effects/test/effect.json",
        br#"{"passes":[{"material":"materials/effects/shake.json"}]}"#,
    );
    let layer = |name: &str| serde_json::json!([{"id":44,"image":"models/image.json","effects":[{"file":"effects/test/effect.json","passes":[{"textures":[name]}]}]}]);
    fixture.scene(layer("_rt_imageLayerComposite_44_a"));
    let scene = PreparedScene::prepare(&fixture.0, None).unwrap();
    assert!(matches!(
        scene.layers[0].stages[1].passes[0].textures[0],
        TextureBinding::LayerComposite
    ));
    for name in [
        "_rt_imageLayerComposite_45_a",
        "_rt_imageLayerComposite_44_b",
        "_alias_unknown",
    ] {
        fixture.scene(layer(name));
        assert!(PreparedScene::prepare(&fixture.0, None).is_err());
    }
}

#[test]
fn history_copy_is_a_command_and_overrides_count_render_passes() {
    let fixture = Fixture::new();
    fixture.shake_shader();
    fixture.write("effects/history/effect.json",br#"{"fbos":[{"name":"history","unique":true},{"name":"scratch"}],"passes":[{"material":"materials/effects/shake.json","target":"scratch","bind":[{"name":"history","index":0}]},{"command":"copy","source":"scratch","target":"history"},{"material":"materials/effects/shake.json","bind":[{"name":"scratch","index":0}]}]}"#);
    fixture.scene(serde_json::json!([{"image":"models/image.json","effects":[{"file":"effects/history/effect.json","passes":[{"constantshadervalues":{"bounds":"0 1"}},{"constantshadervalues":{"bounds":"0 2"}}]}]}]));
    let scene = PreparedScene::prepare(&fixture.0, None).unwrap();
    let stage = &scene.layers[0].stages[1];
    assert_eq!(stage.passes.len(), 3);
    assert!(stage.persistent_targets.contains("history"));
    assert!(
        matches!(&stage.passes[0].textures[0],TextureBinding::History(name) if name=="history")
    );
    assert!(
        matches!(&stage.passes[1].kind,PreparedPassKind::Copy{source:TextureBinding::Named(name)} if name=="scratch")
    );
    assert!(stage.passes[1].shader.is_none());
    assert_eq!(stage.passes[2].constants["g_Bounds"], [0.0, 2.0]);
    for definition in [
        serde_json::json!({"fbos":[{"name":"history","unique":true}],"passes":[{"command":"copy","source":"history","target":"history"},{"material":"materials/effects/shake.json"}]}),
        serde_json::json!({"fbos":[{"name":"scratch"}],"passes":[{"material":"materials/effects/shake.json","bind":[{"name":"scratch","index":0}]}]}),
        serde_json::json!({"fbos":[{"name":"history","unique":"yes"}],"passes":[{"material":"materials/effects/shake.json"}]}),
        serde_json::json!({"fbos":[{"name":"history","unique":true}],"passes":[{"command":"copy","source":"previous","target":"history","script":"x"},{"material":"materials/effects/shake.json"}]}),
    ] {
        fixture.write(
            "effects/history/effect.json",
            &serde_json::to_vec(&definition).unwrap(),
        );
        assert!(PreparedScene::prepare(&fixture.0, None).is_err());
    }
}

#[test]
fn model_crop_metadata_and_sampling_flags_survive_preparation() {
    let fixture = Fixture::new();
    fixture.write("models/image.json",br#"{"autosize":true,"cropoffset":"7 0","nopadding":true,"material":"materials/image.json"}"#);
    fixture.scene(serde_json::json!([{"image":"models/image.json","clampuvs":true}]));
    let scene = PreparedScene::prepare(&fixture.0, None).unwrap();
    assert_eq!(scene.layers[0].crop_offset, [7.0, 0.0]);
    assert!(scene.layers[0].no_padding && scene.layers[0].clamp_uvs);
}

#[test]
fn flat_uniform_quad_preserves_color_without_guessing_vertex_attributes() {
    let fixture = Fixture::new();
    fixture.write(
        "models/image.json",
        br#"{"solidlayer":true,"material":"materials/image.json"}"#,
    );
    fixture.write(
        "materials/image.json",
        br#"{"passes":[{"shader":"flat","blending":"translucent"}]}"#,
    );
    fixture.scene(serde_json::json!([{"image":"models/image.json","size":"16 9","scale":"240 240 1","color":"0 0 0"}]));
    let scene = PreparedScene::prepare(&fixture.0, None).unwrap();
    assert!(scene.textures.is_empty());
    assert_eq!(scene.layers[0].tint, [0.0, 0.0, 0.0, 1.0]);
    fixture.write(
        "materials/image.json",
        br#"{"passes":[{"shader":"flat","combos":{"VERTEXCOLOR":1}}]}"#,
    );
    assert!(PreparedScene::prepare(&fixture.0, None).is_err());
}

#[test]
fn timed_cpu_source_reuses_owned_encoded_data_and_invalidates_same_length_edits() {
    let fixture = Fixture::new();
    let mut bytes = b"TEXV0005\0TEXI0001\0".to_vec();
    for value in [4_u32, 7, 10, 7, 10, 7, 0] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(b"TEXB0004\0");
    for value in [1_u32, u32::MAX, 0, 1, 10, 7, 0, 0, 96] {
        bytes.extend(value.to_le_bytes());
    }
    let payload_offset = bytes.len();
    bytes.extend([0_u8; 96]);
    bytes.extend(b"TEXS0003\0");
    for value in [1_u32, 2, 2] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(0_u32.to_le_bytes());
    bytes.extend(0.1_f32.to_le_bytes());
    for value in [0_f32, 0., 2., 0., 0., 2.] {
        bytes.extend(value.to_le_bytes());
    }
    fixture.write("materials/image.tex", &bytes);
    fixture.write("materials/image.json",br#"{"passes":[{"shader":"genericimage4","textures":["image"],"combos":{"SPRITESHEET":1}}]}"#);
    let first = PreparedScene::prepare(&fixture.0, None).unwrap();
    let second = PreparedScene::prepare(&fixture.0, None).unwrap();
    let source = first.textures[0].timed.as_ref().unwrap();
    let weak = Arc::downgrade(source);
    assert!(Arc::ptr_eq(
        source,
        second.textures[0].timed.as_ref().unwrap()
    ));
    assert!(first.textures[0].rgba().is_empty());
    assert!(first.summary().animated);
    bytes[payload_offset] = 255;
    fixture.write("materials/image.tex", &bytes);
    let changed = PreparedScene::prepare(&fixture.0, None).unwrap();
    assert!(!Arc::ptr_eq(
        source,
        changed.textures[0].timed.as_ref().unwrap()
    ));
    assert_ne!(
        first.textures[0].fingerprint,
        changed.textures[0].fingerprint
    );
    drop(first);
    drop(second);
    assert!(weak.upgrade().is_none());
}

#[test]
fn verified_static_image4_flags_keep_alpha_and_unknown_flags_fail_closed() {
    let fixture = Fixture::new();
    let mut bytes = std::fs::read(fixture.0.join("materials/image.tex")).unwrap();
    bytes[22..26].copy_from_slice(&3_u32.to_le_bytes());
    *bytes.last_mut().unwrap() = 127;
    fixture.write("materials/image.tex", &bytes);
    let scene = PreparedScene::prepare(&fixture.0, None).unwrap();
    assert_eq!(scene.textures[0].source_flags, 3);
    assert_eq!(scene.textures[0].rgba(), [255, 0, 0, 127]);
    bytes[22..26].copy_from_slice(&8_u32.to_le_bytes());
    fixture.write("materials/image.tex", &bytes);
    let error = PreparedScene::prepare(&fixture.0, None).unwrap_err();
    assert_eq!(unsupported_feature_code(&error), Some("shader"));
    assert!(error.to_string().contains("flags 8"));
}
