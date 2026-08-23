package app.dure.sijang;

import android.app.Activity;
import android.content.Context;
import android.util.Log;
import androidx.window.embedding.RuleController;

public class ActivityEmbeddingInitializer {
    private static final String TAG = "ActivityEmbedding";
    private static boolean initialized = false;

    public static void initialize(Activity activity) {
        if (initialized) {
            Log.i(TAG, "Already initialized");
            return;
        }

        try {
            Log.i(TAG, "Initializing Activity Embedding with RuleController");

            // Get RuleController instance
            RuleController ruleController = RuleController.getInstance(activity);

            // Parse and load rules from XML configuration
            ruleController.setRules(RuleController.parseRules(activity, R.xml.main_split_config));

            initialized = true;
            Log.i(TAG, "Activity Embedding initialized successfully from XML config");
        } catch (Exception e) {
            Log.e(TAG, "Failed to initialize Activity Embedding", e);
        }
    }
}
