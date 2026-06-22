use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::prelude::*;
use predicates::prelude::*;

fn move_cmd() -> assert_cmd::Command {
    cargo_bin_cmd!("move")
}

#[test]
fn renames_a_file_and_removes_the_source() {
    let temp = assert_fs::TempDir::new().unwrap();
    let source = temp.child("a.txt");
    let dest = temp.child("b.txt");
    source.write_str("hello").unwrap();

    move_cmd()
        .arg(source.path())
        .arg(dest.path())
        .assert()
        .success();

    source.assert(predicate::path::missing());
    dest.assert("hello");
}

#[test]
fn moves_into_an_existing_directory() {
    let temp = assert_fs::TempDir::new().unwrap();
    let source = temp.child("a.txt");
    let dest_dir = temp.child("dest");
    source.write_str("hello").unwrap();
    dest_dir.create_dir_all().unwrap();

    move_cmd()
        .arg(source.path())
        .arg(dest_dir.path())
        .assert()
        .success();

    source.assert(predicate::path::missing());
    dest_dir.child("a.txt").assert("hello");
}

#[test]
fn moves_multiple_sources_into_a_directory() {
    let temp = assert_fs::TempDir::new().unwrap();
    let one = temp.child("one.txt");
    let two = temp.child("two.txt");
    let dest_dir = temp.child("dest");
    one.write_str("1").unwrap();
    two.write_str("2").unwrap();
    dest_dir.create_dir_all().unwrap();

    move_cmd()
        .arg(one.path())
        .arg(two.path())
        .arg(dest_dir.path())
        .assert()
        .success();

    dest_dir.child("one.txt").assert("1");
    dest_dir.child("two.txt").assert("2");
    one.assert(predicate::path::missing());
}

#[test]
fn target_directory_flag_moves_sources_into_dir() {
    let temp = assert_fs::TempDir::new().unwrap();
    let one = temp.child("one.txt");
    let two = temp.child("two.txt");
    let dest_dir = temp.child("dest");
    one.write_str("1").unwrap();
    two.write_str("2").unwrap();
    dest_dir.create_dir_all().unwrap();

    move_cmd()
        .arg("-t")
        .arg(dest_dir.path())
        .arg(one.path())
        .arg(two.path())
        .assert()
        .success();

    dest_dir.child("one.txt").assert("1");
    dest_dir.child("two.txt").assert("2");
}

#[test]
fn renames_a_directory() {
    let temp = assert_fs::TempDir::new().unwrap();
    let source = temp.child("src");
    let dest = temp.child("dst");
    source.child("inner/f.txt").write_str("deep").unwrap();

    move_cmd()
        .arg(source.path())
        .arg(dest.path())
        .assert()
        .success();

    source.assert(predicate::path::missing());
    dest.child("inner/f.txt").assert("deep");
}

#[test]
fn no_clobber_leaves_the_destination_untouched() {
    let temp = assert_fs::TempDir::new().unwrap();
    let source = temp.child("a.txt");
    let dest = temp.child("b.txt");
    source.write_str("new").unwrap();
    dest.write_str("old").unwrap();

    move_cmd()
        .arg("-n")
        .arg(source.path())
        .arg(dest.path())
        .assert()
        .success();

    dest.assert("old");
    source.assert(predicate::path::exists());
}

#[test]
fn force_overwrites_the_destination() {
    let temp = assert_fs::TempDir::new().unwrap();
    let source = temp.child("a.txt");
    let dest = temp.child("b.txt");
    source.write_str("new").unwrap();
    dest.write_str("old").unwrap();

    move_cmd()
        .arg("-f")
        .arg(source.path())
        .arg(dest.path())
        .assert()
        .success();

    dest.assert("new");
    source.assert(predicate::path::missing());
}

#[test]
fn interactive_declined_keeps_the_destination() {
    let temp = assert_fs::TempDir::new().unwrap();
    let source = temp.child("a.txt");
    let dest = temp.child("b.txt");
    source.write_str("new").unwrap();
    dest.write_str("old").unwrap();

    move_cmd()
        .arg("-i")
        .arg(source.path())
        .arg(dest.path())
        .write_stdin("n\n")
        .assert()
        .success();

    dest.assert("old");
    source.assert(predicate::path::exists());
}

#[test]
fn interactive_accepted_overwrites_the_destination() {
    let temp = assert_fs::TempDir::new().unwrap();
    let source = temp.child("a.txt");
    let dest = temp.child("b.txt");
    source.write_str("new").unwrap();
    dest.write_str("old").unwrap();

    move_cmd()
        .arg("-i")
        .arg(source.path())
        .arg(dest.path())
        .write_stdin("y\n")
        .assert()
        .success();

    dest.assert("new");
    source.assert(predicate::path::missing());
}

#[test]
fn backup_preserves_the_old_destination() {
    let temp = assert_fs::TempDir::new().unwrap();
    let source = temp.child("a.txt");
    let dest = temp.child("b.txt");
    source.write_str("new").unwrap();
    dest.write_str("old").unwrap();

    move_cmd()
        .arg("-b")
        .arg(source.path())
        .arg(dest.path())
        .assert()
        .success();

    dest.assert("new");
    temp.child("b.txt~").assert("old");
}

#[test]
fn verbose_reports_the_rename() {
    let temp = assert_fs::TempDir::new().unwrap();
    let source = temp.child("a.txt");
    let dest = temp.child("b.txt");
    source.write_str("x").unwrap();

    move_cmd()
        .arg("-v")
        .arg(source.path())
        .arg(dest.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("renamed"));
}

#[test]
fn exclude_keeps_excluded_files_in_the_source() {
    let temp = assert_fs::TempDir::new().unwrap();
    let source = temp.child("src");
    source.child("keep.log").write_str("keep").unwrap();
    source.child("data.txt").write_str("move").unwrap();
    let dest = temp.child("dst");

    move_cmd()
        .arg("-e")
        .arg("*.log")
        .arg(source.path())
        .arg(dest.path())
        .assert()
        .success();

    // The excluded file is left behind in the source; everything else moved.
    source.child("keep.log").assert("keep");
    source.child("data.txt").assert(predicate::path::missing());
    dest.child("data.txt").assert("move");
    dest.child("keep.log").assert(predicate::path::missing());
}

#[test]
fn missing_source_fails() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dest = temp.child("b.txt");

    move_cmd()
        .arg(temp.child("does-not-exist.txt").path())
        .arg(dest.path())
        .assert()
        .failure();
}
