package app.dure.sijang;

import android.app.Application;
import android.content.ComponentName;
import android.util.Log;
import androidx.window.embedding.RuleController;
import androidx.window.embedding.SplitPairRule;
import androidx.window.embedding.SplitPairFilter;
import androidx.window.embedding.SplitRule;
import java.util.Collections;
import java.util.HashSet;

public class DureSijangApplication extends Application {
    private static final String TAG = "DureSijangApp";
    private static DureSijangApplication instance;

    @Override
    public void onCreate() {
        super.onCreate();
        instance = this;

        Log.i(TAG, "Application onCreate - Initializing Activity Embedding");

        try {
            // Initialize Activity Embedding at app startup
            RuleController ruleController = RuleController.getInstance(this);
            ruleController.setRules(RuleController.parseRules(this, R.xml.main_split_config));

            Log.i(TAG, "Activity Embedding rules loaded at application startup");
        } catch (Exception e) {
            Log.e(TAG, "Failed to initialize Activity Embedding", e);
        }
    }

    public static DureSijangApplication getInstance() {
        return instance;
    }

    public static void updateSplitRatio(float splitRatio) {
        if (instance == null) return;
        Log.i(TAG, "Updating split ratio to: " + splitRatio);
        try {
            RuleController rc = RuleController.getInstance(instance);
            HashSet<SplitPairFilter> filters = new HashSet<>();
            filters.add(new SplitPairFilter(
                new ComponentName(instance, "android.app.NativeActivity"),
                new ComponentName(instance, WebViewActivity.class), null));

            SplitPairRule rule = new SplitPairRule.Builder(filters)
                .setDefaultSplitAttributes(new androidx.window.embedding.SplitAttributes.Builder()
                    .setSplitType(androidx.window.embedding.SplitAttributes.SplitType.ratio(splitRatio))
                    .setLayoutDirection(androidx.window.embedding.SplitAttributes.LayoutDirection.TOP_TO_BOTTOM)
                    .build())
                .setMinWidthDp(0).setMinHeightDp(0).setMinSmallestWidthDp(0)
                .setMaxAspectRatioInPortrait(androidx.window.embedding.EmbeddingAspectRatio.ALWAYS_ALLOW)
                .setMaxAspectRatioInLandscape(androidx.window.embedding.EmbeddingAspectRatio.ALWAYS_ALLOW)
                .setFinishPrimaryWithSecondary(SplitRule.FinishBehavior.NEVER)
                .setFinishSecondaryWithPrimary(SplitRule.FinishBehavior.ALWAYS)
                .setClearTop(false).build();
            rc.setRules(Collections.singleton(rule));
        } catch (Exception e) {
            Log.e(TAG, "Failed to update split", e);
        }
    }
}
