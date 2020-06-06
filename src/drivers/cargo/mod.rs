struct CargoToml {
    // https://doc.rust-lang.org/cargo/reference/manifest.html 2020/06/03
    package: Package,
    lib: Lib,
    bin: Option<Vec<Bin>>,
    example: Option<Vec<Example>>,
    test: Option<Vec<Test>>,
    bench: Option<Vec<Bench>>,
    // https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html
    // @TODO: what about [dependencies.baz]?
    dependencies: Option<Vec<Dependency>>,
    dev_dependencies: Option<Vec<Dependency>>,
    build_dependencies: Option<Vec<Dependency>>,
    target: Option<Vec<Target>>,
    badges: Option<Vec<Badge>>,
    // https://doc.rust-lang.org/cargo/reference/features.html
    features: Option<Vec<Feature>>,
    // https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html
    patch: Option<Vec<Patch>>,
    replace: Option<Vec<Replace>>,
    paths: Option<Vec<Path>>,
    // https://doc.rust-lang.org/cargo/reference/profiles.html
    profile: Option<Vec<Profile>>,
    // https://doc.rust-lang.org/cargo/reference/workspaces.html
    workspace: Option<Vec<Workspace>>,
}

type Package = String;
type Lib = String;
type Path = String;
type Bin = String;
type Example = String;
type Test = String;
type Bench = String;
type Dependency = String;
type Target = String;
type Badge = String;
type Feature = String;
type Patch = String;
type Replace = String;
type Profile = String;
type Workspace = String;
