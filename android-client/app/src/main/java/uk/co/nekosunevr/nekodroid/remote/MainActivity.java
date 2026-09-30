package uk.co.nekosunevr.nekodroid.remote;

import android.app.Activity;
import android.graphics.Color;
import android.os.Bundle;
import android.view.Gravity;
import android.view.View;
import android.view.ViewGroup;
import android.webkit.WebChromeClient;
import android.webkit.WebSettings;
import android.webkit.WebView;
import android.webkit.WebViewClient;
import android.widget.Button;
import android.widget.EditText;
import android.widget.LinearLayout;
import android.widget.TextView;

public class MainActivity extends Activity {
    private WebView webView;
    private LinearLayout connectPanel;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        showConnectScreen();
    }

    private void showConnectScreen() {
        LinearLayout root = new LinearLayout(this);
        root.setOrientation(LinearLayout.VERTICAL);
        root.setGravity(Gravity.CENTER);
        root.setPadding(dp(20), dp(24), dp(20), dp(24));
        root.setBackgroundColor(Color.rgb(3, 8, 6));

        connectPanel = new LinearLayout(this);
        connectPanel.setOrientation(LinearLayout.VERTICAL);
        connectPanel.setPadding(dp(18), dp(18), dp(18), dp(18));
        connectPanel.setBackgroundColor(Color.rgb(7, 18, 12));

        TextView title = new TextView(this);
        title.setText("NekoDroid Remote");
        title.setTextSize(28);
        title.setTextColor(Color.rgb(233, 255, 243));
        title.setPadding(0, 0, 0, dp(14));

        TextView help = new TextView(this);
        help.setText("Enter your NekoDroid Remote node and invite code.");
        help.setTextColor(Color.rgb(143, 169, 155));
        help.setPadding(0, 0, 0, dp(12));

        EditText server = new EditText(this);
        server.setHint("https://remote.example.com");
        server.setTextColor(Color.WHITE);
        server.setHintTextColor(Color.GRAY);
        server.setSingleLine(true);

        EditText invite = new EditText(this);
        invite.setHint("Invite code");
        invite.setTextColor(Color.WHITE);
        invite.setHintTextColor(Color.GRAY);
        invite.setSingleLine(true);

        Button connect = new Button(this);
        connect.setText("Connect");
        connect.setOnClickListener(v -> {
            String base = server.getText().toString().trim();
            String code = invite.getText().toString().trim();
            if (base.isEmpty() || code.isEmpty()) {
                help.setText("Remote node URL and invite code are required.");
                return;
            }
            if (!base.startsWith("http://") && !base.startsWith("https://")) {
                base = "https://" + base;
            }
            while (base.endsWith("/")) {
                base = base.substring(0, base.length() - 1);
            }
            openRemote(base + "/?invite=" + android.net.Uri.encode(code));
        });

        connectPanel.addView(title);
        connectPanel.addView(help);
        connectPanel.addView(server, matchWrap());
        connectPanel.addView(invite, matchWrap());
        connectPanel.addView(connect, matchWrap());

        root.addView(connectPanel, new LinearLayout.LayoutParams(
            ViewGroup.LayoutParams.MATCH_PARENT,
            ViewGroup.LayoutParams.WRAP_CONTENT
        ));
        setContentView(root);
    }

    private void openRemote(String url) {
        webView = new WebView(this);
        webView.setBackgroundColor(Color.BLACK);

        WebSettings settings = webView.getSettings();
        settings.setJavaScriptEnabled(true);
        settings.setDomStorageEnabled(true);
        settings.setMediaPlaybackRequiresUserGesture(false);
        settings.setBuiltInZoomControls(false);
        settings.setDisplayZoomControls(false);
        settings.setUseWideViewPort(true);
        settings.setLoadWithOverviewMode(true);

        webView.setWebViewClient(new WebViewClient());
        webView.setWebChromeClient(new WebChromeClient());
        webView.loadUrl(url);
        setContentView(webView);
    }

    @Override
    public void onBackPressed() {
        if (webView != null && webView.canGoBack()) {
            webView.goBack();
        } else if (webView != null) {
            webView.destroy();
            webView = null;
            showConnectScreen();
        } else {
            super.onBackPressed();
        }
    }

    private LinearLayout.LayoutParams matchWrap() {
        LinearLayout.LayoutParams params = new LinearLayout.LayoutParams(
            ViewGroup.LayoutParams.MATCH_PARENT,
            ViewGroup.LayoutParams.WRAP_CONTENT
        );
        params.setMargins(0, dp(6), 0, dp(6));
        return params;
    }

    private int dp(int value) {
        return Math.round(value * getResources().getDisplayMetrics().density);
    }
}
