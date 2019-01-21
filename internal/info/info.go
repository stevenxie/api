package info

var (
	// Version is the program version, set during compile time using:
	//   -ldflags -X github.com/take-care/api/internal.Version=$(VERSION)
	Version = "unset"

	// Namespace is the project namespace, to be used as prefixes for environment
	// variables, etc.
	Namespace = "api"
)
