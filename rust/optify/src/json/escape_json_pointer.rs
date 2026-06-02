macro_rules! escape_json_pointer {
    ($id:ident) => {
        use ::cow_utils::CowUtils;
        let $id = $id.cow_replace("~", "~0");
        let $id = $id.cow_replace("/", "~1");
    };
}

pub(crate) use escape_json_pointer;
