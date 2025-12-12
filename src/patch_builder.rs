use brdb::pending::BrPendingFs;

pub fn create_components_patch(mut chunk_files: Vec<(String, BrPendingFs)>, mut brick_grids_folder: Vec<(String, BrPendingFs)>) -> BrPendingFs {
    brick_grids_folder.push((
        String::from("1"),
        BrPendingFs::Folder(Some(vec![(
            "Components".to_string(),
            BrPendingFs::Folder(Some(chunk_files)),
        )]))));
    let components_patch = BrPendingFs::Root(vec![(
        "World".to_owned(),
        BrPendingFs::Folder(Some(vec![(
            "0".to_string(),
            BrPendingFs::Folder(Some(vec![(
                "Bricks".to_string(),
                BrPendingFs::Folder(Some(vec![(
                    "Grids".to_string(),
                    BrPendingFs::Folder(Some(brick_grids_folder)),
                )])),
            )])),
        )])),
    )]);
    components_patch
}