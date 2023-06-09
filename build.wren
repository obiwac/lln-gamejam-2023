// dependencies

Deps.git_inherit("https://github.com/inobulles/aqua-unix")
Deps.git_inherit("https://github.com/inobulles/iar")

// shader compilation

var glsl_src = File.list("src/shaders")
	.where { |path| path.endsWith(".vert") || path.endsWith(".frag") }

glsl_src
	.each { |path| File.exec("glslc", [path, "-o", "%(path).spv"]) }

// compilation

var rustc = RustC.new()

rustc.add_dep("ash", "https://github.com/obiwac/ash-aqua")
rustc.add_dep("aqua", "https://github.com/inobulles/aqua-rs", ["vk"])

var src = ["src/main.rs"]

src
	.each { |path| rustc.compile(path) }

// link program

var linker = Linker.new()

if (Meta.os().contains("Linux")) {
	linker.link(src.toList, ["std-2908e577647e150b"], "main", true)
}

if (Meta.os().contains("FreeBSD")) {
	linker.link(src.toList, ["std-7c7f3bd22bdaa9dd"], "main", true)
}


// running

class Runner {
	static run(args) { File.exec("kos", ["--boot", "default.zpk"]) }
}

// installation map

var entry = "lln-gamejam-2023"

var install = {
	"main": entry,
}

// packaging

var pkg = Package.new(entry)

pkg.unique = "lln.gamejam.2023"
pkg.name = "Louvain-li-Nux Gamejam 2023"
pkg.description = "Submission for the 2023 Louvain-li-Nux gamejam"
pkg.version = "0.1.0"
pkg.author = "@alexisloic21 @obiwac @akialess"
pkg.organization = "Louvain-li-Nux"
pkg.www = "https://github.com/obiwac/lln-gamejam-2023"

var packages = {
	"default": pkg,
}

// testing

class Tests {
}

var tests = []
