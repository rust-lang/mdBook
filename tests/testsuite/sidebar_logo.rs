use crate::prelude::*;

#[test]
fn bad_logo_path() {
    BookTest::from_dir("sidebar/logo/not_found").run("build", |cmd| {
        cmd.expect_failure();
        cmd.expect_stderr(
            "ERROR invalid value for `logo`: [ROOT]/src/does/not/exist does not exist\n",
        );
    });
}
