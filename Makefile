# HP-48 Web Emulator - Emscripten Build
CC = emcc
SRCDIR = src/emu
WEBDIR = web

SOURCES = $(SRCDIR)/main_wasm.c \
          $(SRCDIR)/actions.c \
          $(SRCDIR)/binio.c \
          $(SRCDIR)/device.c \
          $(SRCDIR)/emulate.c \
          $(SRCDIR)/errors.c \
          $(SRCDIR)/init.c \
          $(SRCDIR)/lcd.c \
          $(SRCDIR)/memory.c \
          $(SRCDIR)/options.c \
          $(SRCDIR)/register.c \
          $(SRCDIR)/resources.c \
          $(SRCDIR)/romio.c \
          $(SRCDIR)/rpl.c \
          $(SRCDIR)/serial.c \
          $(SRCDIR)/timer.c \
          $(SRCDIR)/x48_web.c

CFLAGS = -O2 -DNDEBUG \
         -Wno-format-security \
         -Wno-dangling-else \
         -I$(SRCDIR) \
         -sALLOW_MEMORY_GROWTH=1

LDFLAGS = -sEXPORTED_FUNCTIONS='["_main","_push_key_event","_get_display_buffer","_get_display_width","_get_display_height","_is_display_dirty","_clear_display_dirty","_get_annunciator_state","_get_speaker_frequency","_web_save_state","_start_emulation","_persist_ready"]' \
          -sEXPORTED_RUNTIME_METHODS='["ccall","cwrap","HEAPU8"]' \
          -sFORCE_FILESYSTEM=1 -lidbfs.js \
          --preload-file assets@/assets \
          -o $(WEBDIR)/hp48_emu.js

all: $(WEBDIR)/hp48.js $(WEBDIR)/hp48_emu.js

# TypeScript -> JS via esbuild
$(WEBDIR)/hp48.js: $(WEBDIR)/hp48.ts
	npx esbuild $< --bundle --format=iife --outfile=$@

# C -> WASM via Emscripten
$(WEBDIR)/hp48_emu.js: $(SOURCES)
	@mkdir -p $(WEBDIR)
	$(CC) $(CFLAGS) $(SOURCES) $(LDFLAGS)

typecheck:
	npx tsc --project $(WEBDIR)/tsconfig.json --noEmit

lint:
	npx eslint $(WEBDIR)/hp48.ts

check: typecheck lint

clean:
	rm -f $(WEBDIR)/hp48_emu.js $(WEBDIR)/hp48_emu.wasm $(WEBDIR)/hp48_emu.data $(WEBDIR)/hp48.js

.PHONY: all clean typecheck lint check
