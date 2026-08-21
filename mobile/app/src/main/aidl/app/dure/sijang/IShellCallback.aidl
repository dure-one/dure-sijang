package app.dure.sijang;

interface IShellCallback {
    void onOutput(String line);
    void onComplete(int exitCode);
}
