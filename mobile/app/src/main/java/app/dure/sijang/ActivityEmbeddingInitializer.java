package app.dure.sijang;

import android.app.Activity;
import android.util.Log;

public class ActivityEmbeddingInitializer {
    private static final String TAG = "ActivityEmbedding";
    private static boolean initialized = false;

    public static void initialize(Activity activity) {
        if (initialized) {
            Log.i(TAG, "Already initialized");
            return;
        }

        try {
            Log.i(TAG, "Initializing Activity Embedding with programmatic split rule");

            // Reinstall the divider-aware rule after the XML metadata has been processed.
            DureSijangApplication.updateSplitRatio(0.1f);

            initialized = true;
            Log.i(TAG, "Activity Embedding initialized successfully");
        } catch (Exception e) {
            Log.e(TAG, "Failed to initialize Activity Embedding", e);
        }
    }
}
