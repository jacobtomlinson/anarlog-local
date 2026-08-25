import {
  KeyboardAvoidingView,
  Modal,
  Platform,
  Pressable,
  ScrollView,
  StyleSheet,
  Text,
  View,
} from "react-native";

import { useAuth } from "@/auth/context";
import { Button } from "@/components/ui/button";
import {
  Colors,
  CornerCurve,
  Radius,
  Spacing,
  Typography,
} from "@/constants/theme";

export function ProfileSheet({
  visible,
  onClose,
}: {
  visible: boolean;
  onClose: () => void;
}) {
  const auth = useAuth();
  const planLabel = auth.bypass
    ? "Local dev"
    : auth.billing.plan === "trial"
      ? `Trial · ${auth.billing.trialDaysRemaining ?? 0}d left`
      : auth.billing.plan === "pro"
        ? "Pro"
        : "Free";

  const close = () => onClose();

  return (
    <Modal
      visible={visible}
      transparent
      animationType="slide"
      onRequestClose={close}
    >
      <KeyboardAvoidingView
        behavior={Platform.OS === "ios" ? "padding" : undefined}
        style={styles.backdrop}
      >
        <Pressable style={styles.backdropPressable} onPress={close}>
          <Pressable style={styles.sheet} onPress={(e) => e.stopPropagation()}>
            <View style={styles.handle} />
            <ScrollView contentContainerStyle={styles.content}>
              <View style={styles.row}>
                <Text style={styles.email} numberOfLines={1}>
                  {auth.bypass
                    ? "Not signed in"
                    : (auth.session?.user.email ?? "")}
                </Text>
                <View style={styles.planChip}>
                  <Text style={styles.planLabel}>{planLabel}</Text>
                </View>
              </View>

              {!auth.bypass && (
                <Button
                  label="Sign out"
                  onPress={() => {
                    close();
                    void auth.signOut();
                  }}
                  size="small"
                  style={styles.signOut}
                  variant="ghost"
                />
              )}
            </ScrollView>
          </Pressable>
        </Pressable>
      </KeyboardAvoidingView>
    </Modal>
  );
}

const styles = StyleSheet.create({
  backdrop: {
    flex: 1,
  },
  backdropPressable: {
    flex: 1,
    justifyContent: "flex-end",
    backgroundColor: Colors.scrim,
  },
  sheet: {
    maxHeight: "88%",
    backgroundColor: Colors.surface,
    borderTopLeftRadius: Radius.sheet,
    borderTopRightRadius: Radius.sheet,
    paddingHorizontal: Spacing.lg,
    paddingBottom: Spacing.xl,
    paddingTop: Spacing.sm,
  },
  content: {
    paddingBottom: Spacing.md,
  },
  handle: {
    alignSelf: "center",
    width: 40,
    height: 4,
    borderRadius: 2,
    borderCurve: CornerCurve.squircle,
    backgroundColor: Colors.border,
    marginBottom: Spacing.md,
  },
  row: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
    gap: Spacing.sm,
  },
  email: {
    flex: 1,
    ...Typography.section,
    color: Colors.ink,
  },
  planChip: {
    borderWidth: StyleSheet.hairlineWidth,
    borderColor: Colors.border,
    borderRadius: Radius.pill,
    borderCurve: CornerCurve.squircle,
    backgroundColor: Colors.accentSurface,
    paddingHorizontal: Spacing.sm,
    paddingVertical: Spacing.xs,
  },
  planLabel: {
    ...Typography.captionStrong,
    color: Colors.ink,
  },
  signOut: {
    marginTop: Spacing.lg,
    alignSelf: "flex-start",
  },
});
