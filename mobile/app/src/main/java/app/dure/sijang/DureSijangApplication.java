package app.dure.sijang;

import android.app.Application;
import android.content.ComponentName;
import android.util.Log;
import androidx.core.content.ContextCompat;
import androidx.window.core.ExtensionsUtil;
import androidx.window.embedding.DividerAttributes;
import androidx.window.embedding.RuleController;
import androidx.window.embedding.SplitAttributes;
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

        Log.i(TAG, "Application onCreate - Initializing Activity Embedding with expandable panes");

        try {
            // Initialize Activity Embedding at app startup with draggable divider support
            RuleController ruleController = RuleController.getInstance(this);
            ruleController.setRules(createSplitRules(0.3f));

            Log.i(TAG, "Activity Embedding rules loaded with expandable pane support");
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
            rc.setRules(createSplitRules(splitRatio));
        } catch (Exception e) {
            Log.e(TAG, "Failed to update split", e);
        }
    }

    /**
     * Create split rules with expandable pane (draggable divider) support
     */
    private static HashSet<SplitRule> createSplitRules(float splitRatio) {
        HashSet<SplitRule> rules = new HashSet<>();

        // Build SplitAttributes with expandable pane support
        SplitAttributes.Builder splitAttributesBuilder = new SplitAttributes.Builder()
            .setSplitType(SplitAttributes.SplitType.ratio(splitRatio))
            .setLayoutDirection(SplitAttributes.LayoutDirection.TOP_TO_BOTTOM);

        // Check if WindowManager Extensions 6+ is available for draggable divider
        int extensionsVersion = ExtensionsUtil.getSafeVendorApiLevel();
        if (extensionsVersion >= 6) {
            Log.i(TAG, "WindowManager Extensions " + extensionsVersion + " - enabling draggable divider");

            // Add draggable divider attributes
            splitAttributesBuilder.setDividerAttributes(
                new DividerAttributes.DraggableDividerAttributes.Builder()
                    .setColor(ContextCompat.getColor(instance, R.color.divider_color))
                    .setWidthDp(4)
                    .setDragRange(DividerAttributes.DragRange.DRAG_RANGE_SYSTEM_DEFAULT)
                    .build()
            );
        } else {
            Log.i(TAG, "WindowManager Extensions " + extensionsVersion + " - draggable divider requires version 6+ (Android 15+)");
        }

        SplitAttributes splitAttributes = splitAttributesBuilder.build();

        // Create SplitPairFilter for NativeActivity (primary) and WebViewActivity (secondary)
        HashSet<SplitPairFilter> filters = new HashSet<>();
        filters.add(new SplitPairFilter(
            new ComponentName(instance, "android.app.NativeActivity"),
            new ComponentName(instance, WebViewActivity.class),
            null
        ));

        // Build SplitPairRule with expandable pane support
        SplitPairRule splitPairRule = new SplitPairRule.Builder(filters)
            .setDefaultSplitAttributes(splitAttributes)
            .setMinWidthDp(0)
            .setMinHeightDp(0)
            .setMinSmallestWidthDp(0)
            .setMaxAspectRatioInPortrait(androidx.window.embedding.EmbeddingAspectRatio.ALWAYS_ALLOW)
            .setMaxAspectRatioInLandscape(androidx.window.embedding.EmbeddingAspectRatio.ALWAYS_ALLOW)
            .setFinishPrimaryWithSecondary(SplitRule.FinishBehavior.NEVER)
            .setFinishSecondaryWithPrimary(SplitRule.FinishBehavior.ALWAYS)
            .setClearTop(true)
            .build();

        rules.add(splitPairRule);
        return rules;
    }
}
