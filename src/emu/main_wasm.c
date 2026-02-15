#include <stdio.h>
#include <string.h>
#include <sys/stat.h>
#include <emscripten.h>

#include "hp48.h"
#include "hp48_emu.h"
#include "device.h"
#include "timer.h"
#include "resources.h"
#include "romio.h"

/* From main.c globals */
char *progname = "x48";
char *res_name = "x48";
char *res_class = "X48";
int saved_argc = 0;
char **saved_argv = NULL;

saturn_t saturn;
int nb;

char files_path[256];
char rom_filename[256];
char ram_filename[256];
char conf_filename[256];
char port1_filename[256];
char port2_filename[256];

/* Target the HP-48's native Saturn CPU instruction rate.
 * Saturn crystal = 3.6864 MHz, typical ~20 cycles/instruction → ~184K ips.
 * Tuned so that a "440 5 BEEP" lasts approximately 5 wall-clock seconds. */
#define TARGET_IPS 184000.0
#define MAX_INSTRUCTIONS_PER_FRAME 10000

/* Instruction counter for speaker analysis (defined in device.c) */
extern long long speaker_instr_count;

/* Forward declarations */
void frame_callback(void);

/* ---------------------------------------------------------------------------
 * IDBFS persistence helpers (implemented in JS via EM_JS)
 * --------------------------------------------------------------------------- */

/* Mount /persist as IDBFS and sync FROM IndexedDB (populate).
   Calls the C function persist_ready() when done. */
EM_JS(void, mount_idbfs, (void), {
    FS.mkdir('/persist');
    FS.mount(IDBFS, {}, '/persist');
    FS.syncfs(true, function(err) {
        if (err) console.error('IDBFS load error:', err);
        ccall('persist_ready', null, [], []);
    });
});

/* Flush /persist TO IndexedDB */
EM_JS(void, sync_idbfs, (void), {
    FS.syncfs(false, function(err) {
        if (err) console.error('IDBFS save error:', err);
    });
});

/* ---------------------------------------------------------------------------
 * File copy helper: /assets/name -> /persist/name (only if dest missing)
 * --------------------------------------------------------------------------- */

static void copy_file_if_missing(const char *name) {
    char src[256], dst[256];
    struct stat st;
    FILE *fin, *fout;
    char buf[4096];
    size_t n;

    snprintf(src, sizeof(src), "/assets/%s", name);
    snprintf(dst, sizeof(dst), "/persist/%s", name);

    if (stat(dst, &st) == 0)
        return;  /* already exists in persist */

    fin = fopen(src, "rb");
    if (!fin) return;  /* bundled asset missing — nothing to copy */

    fout = fopen(dst, "wb");
    if (!fout) { fclose(fin); return; }

    while ((n = fread(buf, 1, sizeof(buf), fin)) > 0)
        fwrite(buf, 1, n, fout);

    fclose(fin);
    fclose(fout);
    printf("Copied %s -> %s\n", src, dst);
}

/* ---------------------------------------------------------------------------
 * Called by JS once IDBFS sync-from-IndexedDB completes
 * --------------------------------------------------------------------------- */

EMSCRIPTEN_KEEPALIVE
void persist_ready(void) {
    printf("IDBFS ready, initializing emulator...\n");

    /* Copy bundled assets to /persist/ on first run */
    copy_file_if_missing("rom");
    copy_file_if_missing("ram");
    copy_file_if_missing("hp48");

    /* Point emulator at persistent storage */
    strcpy(files_path, "/persist/");
    strcpy(rom_filename, "rom");
    strcpy(ram_filename, "ram");
    strcpy(conf_filename, "hp48");
    strcpy(port1_filename, "port1");
    strcpy(port2_filename, "port2");

    get_resources();

    if (init_emulator() < 0) {
        printf("ERROR: Failed to initialize emulator\n");
        return;
    }

    init_active_stuff();

    set_accesstime();
    start_timer(RUN_TIMER);

    printf("Emulator initialized, starting main loop\n");

    emscripten_set_main_loop(frame_callback, 0, 0);
}

/* ---------------------------------------------------------------------------
 * Frame callback
 * --------------------------------------------------------------------------- */

void frame_callback(void) {
    static double last_time = 0;
    double now = emscripten_get_now();  /* milliseconds */

    if (last_time == 0) {
        last_time = now;
        return;
    }

    double elapsed_ms = now - last_time;
    last_time = now;

    /* Cap elapsed time to avoid huge bursts after tab switch */
    if (elapsed_ms > 100.0) elapsed_ms = 100.0;

    int target = (int)(TARGET_IPS * elapsed_ms / 1000.0);
    if (target > MAX_INSTRUCTIONS_PER_FRAME) target = MAX_INSTRUCTIONS_PER_FRAME;
    if (target < 1) target = 1;

    got_alarm = 1;

    if (saturn_is_shutdown) {
        extern void do_shutdown_check(void);
        do_shutdown_check();
        return;
    }

    for (int i = 0; i < target; i++) {
        speaker_instr_count++;
        step_instruction();
        schedule();
        if (saturn_is_shutdown) break;
    }
}

/* ---------------------------------------------------------------------------
 * main — just mounts IDBFS, actual init happens in persist_ready()
 * --------------------------------------------------------------------------- */

int main(int argc, char **argv) {
    (void)argc; (void)argv;
    printf("HP-48 Web Emulator starting...\n");
    mount_idbfs();
    return 0;
}

/* ---------------------------------------------------------------------------
 * Exported functions
 * --------------------------------------------------------------------------- */

EMSCRIPTEN_KEEPALIVE
void web_save_state(void) {
    write_files();
    sync_idbfs();
}

EMSCRIPTEN_KEEPALIVE
void start_emulation(void) {
    /* Called from JS after Module is ready, if needed */
}
