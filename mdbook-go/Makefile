# Local dev/test Makefile for doclens (mdbook-go).
#
#   make build               build bin/doclens with version injected
#   make build VERSION=v1.0.0   override the injected version
#   make version             build and print version (verify -X works)
#   make serve               build + serve the test book
#                            (BOOK=path, PORT=port to override)
#   make dev                 debug build: no strip, no -trimpath
#   make test                run Go tests
#   make clean               remove the binary

BINARY  := doclens
OUT     := bin
PKG     := mdbook-go/cmd/doclens
VER_PKG := mdbook-go/pkg/cmd/version

VERSION ?= 0.1.0
BOOK    ?= tests/basic
PORT    ?= 3000

# release-style flags (same as the one-off build command)
LDFLAGS := -buildid= -s -w -X $(VER_PKG).Version=$(VERSION)

.PHONY: build version serve dev test clean

build:
	go build -trimpath -ldflags="$(LDFLAGS)" -o $(OUT)/$(BINARY) $(PKG)

# quickly verify the -X injection: prints e.g. "doclens v1.0.0 go1.26.4 darwin/arm64"
version: build
	$(OUT)/$(BINARY) version

serve: build
	$(OUT)/$(BINARY) serve --dir $(BOOK) --port $(PORT)

# debug build: keep symbols + local file paths for stack traces / dlv
dev:
	go build -ldflags="-X $(VER_PKG).Version=$(VERSION)" -o $(OUT)/$(BINARY) $(PKG)

test:
	go test ./...

clean:
	rm -f $(OUT)/$(BINARY)
