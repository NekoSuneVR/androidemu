package uk.co.nekosunevr.nekodroid.bridge;

import android.accessibilityservice.AccessibilityService;
import android.accessibilityservice.GestureDescription;
import android.content.*;
import android.graphics.Path;
import android.os.Bundle;
import android.view.accessibility.AccessibilityEvent;
import java.util.*;

public class NekoBridgeAccessibilityService extends AccessibilityService {
  private BroadcastReceiver receiver;

  @Override public void onServiceConnected() {
    receiver = new BroadcastReceiver() {
      @Override public void onReceive(Context context, Intent intent) {
        String action = intent.getAction();
        if (action == null) return;
        if (action.endsWith(".MULTITOUCH")) handleMulti(intent);
        else if (action.endsWith(".PINCH")) handlePinch(intent);
      }
    };
    IntentFilter filter = new IntentFilter();
    filter.addAction("uk.co.nekosunevr.nekodroid.bridge.MULTITOUCH");
    filter.addAction("uk.co.nekosunevr.nekodroid.bridge.PINCH");
    registerReceiver(receiver, filter, Context.RECEIVER_EXPORTED);
  }

  private void handleMulti(Intent intent) {
    String value = intent.getStringExtra("points");
    int duration = Math.max(20, Math.min(5000, intent.getIntExtra("duration", 300)));
    if (value == null) return;
    GestureDescription.Builder builder = new GestureDescription.Builder();
    int count=0;
    for (String point : value.split(";")) {
      String[] xy=point.split(",");
      if (xy.length!=2 || count++>=10) continue;
      try {
        float x=Float.parseFloat(xy[0]), y=Float.parseFloat(xy[1]);
        Path p=new Path(); p.moveTo(x,y); p.lineTo(x,y);
        builder.addStroke(new GestureDescription.StrokeDescription(p,0,duration));
      } catch (NumberFormatException ignored) {}
    }
    if (count>=2) dispatchGesture(builder.build(),null,null);
  }

  private void handlePinch(Intent intent) {
    float cx=intent.getIntExtra("cx",540), cy=intent.getIntExtra("cy",1200);
    float from=intent.getIntExtra("fromRadius",200), to=intent.getIntExtra("toRadius",80);
    int duration=Math.max(20,Math.min(5000,intent.getIntExtra("duration",350)));
    GestureDescription.Builder builder=new GestureDescription.Builder();
    for (int direction : new int[]{-1,1}) {
      Path p=new Path();
      p.moveTo(cx + direction*from, cy);
      p.lineTo(cx + direction*to, cy);
      builder.addStroke(new GestureDescription.StrokeDescription(p,0,duration));
    }
    dispatchGesture(builder.build(),null,null);
  }

  @Override public void onAccessibilityEvent(AccessibilityEvent event) {}
  @Override public void onInterrupt() {}
  @Override public void onDestroy() {
    if (receiver != null) unregisterReceiver(receiver);
    super.onDestroy();
  }
}
