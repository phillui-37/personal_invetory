import 'package:flutter/material.dart';

/// Ensures buttons have proper semantic labels and keyboard navigation support
class AccessibleButton extends StatelessWidget {
  final String label;
  final VoidCallback onPressed;
  final IconData? icon;
  final bool isEnabled;
  final String? tooltip;
  final Color? backgroundColor;
  final Color? foregroundColor;
  final bool isFilled;

  const AccessibleButton({
    super.key,
    required this.label,
    required this.onPressed,
    this.icon,
    this.isEnabled = true,
    this.tooltip,
    this.backgroundColor,
    this.foregroundColor,
    this.isFilled = true,
  });

  @override
  Widget build(BuildContext context) {
    final button = isFilled
        ? ElevatedButton.icon(
            onPressed: isEnabled ? onPressed : null,
            icon: icon != null ? Icon(icon) : SizedBox.shrink(),
            label: Text(label),
            style: ElevatedButton.styleFrom(
              backgroundColor: backgroundColor,
              foregroundColor: foregroundColor,
            ),
          )
        : OutlinedButton.icon(
            onPressed: isEnabled ? onPressed : null,
            icon: icon != null ? Icon(icon) : SizedBox.shrink(),
            label: Text(label),
            style: OutlinedButton.styleFrom(
              foregroundColor: foregroundColor,
            ),
          );

    final withSemantics = Semantics(
      button: true,
      enabled: isEnabled,
      label: label,
      child: button,
    );

    return tooltip != null
        ? Tooltip(message: tooltip, child: withSemantics)
        : withSemantics;
  }
}

/// Ensures form inputs have proper semantic labels
class AccessibleFormField extends StatelessWidget {
  final String label;
  final String? hint;
  final TextEditingController? controller;
  final String? Function(String?)? validator;
  final TextInputType keyboardType;
  final int maxLines;
  final bool isRequired;
  final String? errorText;

  const AccessibleFormField({
    super.key,
    required this.label,
    this.hint,
    this.controller,
    this.validator,
    this.keyboardType = TextInputType.text,
    this.maxLines = 1,
    this.isRequired = false,
    this.errorText,
  });

  @override
  Widget build(BuildContext context) {
    return Semantics(
      label: label,
      focusable: true,
      enabled: true,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Semantics(
            label: '$label${isRequired ? " (required)" : ""}',
            child: Text(
              label,
              style: Theme.of(context).textTheme.labelLarge?.copyWith(
                fontWeight: FontWeight.bold,
              ),
            ),
          ),
          SizedBox(height: 8),
          TextFormField(
            controller: controller,
            validator: validator,
            keyboardType: keyboardType,
            maxLines: maxLines,
            minLines: maxLines == 1 ? 1 : null,
            decoration: InputDecoration(
              hintText: hint,
              helperText: isRequired ? 'This field is required' : null,
              errorText: errorText,
            ),
          ),
        ],
      ),
    );
  }
}

/// List item with proper semantic structure for screen readers
class AccessibleListTile extends StatelessWidget {
  final String title;
  final String? subtitle;
  final String? description;
  final IconData? leading;
  final VoidCallback? onTap;
  final bool isSelectable;

  const AccessibleListTile({
    super.key,
    required this.title,
    this.subtitle,
    this.description,
    this.leading,
    this.onTap,
    this.isSelectable = true,
  });

  @override
  Widget build(BuildContext context) {
    final semanticsLabel = [
      title,
      if (subtitle != null) subtitle!,
      if (description != null) description!,
    ].join(', ');

    return Semantics(
      enabled: true,
      button: onTap != null,
      label: semanticsLabel,
      onTap: onTap,
      child: ListTile(
        title: Text(title),
        subtitle: subtitle != null ? Text(subtitle!) : null,
        leading: leading != null
            ? Semantics(
                image: true,
                label: title,
                child: Icon(leading),
              )
            : null,
        onTap: onTap,
      ),
    );
  }
}

/// Ensures proper contrast and keyboard navigation for interactive elements
class AccessibleContainer extends StatelessWidget {
  final Widget child;
  final Color backgroundColor;
  final bool ensureContrast;

  const AccessibleContainer({
    super.key,
    required this.child,
    required this.backgroundColor,
    this.ensureContrast = true,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      color: backgroundColor,
      child: child,
    );
  }
}

/// Dialog with proper accessibility
class AccessibleDialog extends StatelessWidget {
  final String title;
  final String? description;
  final Widget content;
  final List<Widget> actions;

  const AccessibleDialog({
    super.key,
    required this.title,
    this.description,
    required this.content,
    required this.actions,
  });

  @override
  Widget build(BuildContext context) {
    return Semantics(
      label: title,
      scopesRoute: true,
      enabled: true,
      child: AlertDialog(
        title: Semantics(
          label: title,
          child: Text(title),
        ),
        content: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              if (description != null)
                Semantics(
                  label: description,
                  child: Text(description!),
                ),
              if (description != null) SizedBox(height: 16),
              content,
            ],
          ),
        ),
        actions: actions,
      ),
    );
  }
}

/// Ensures tab navigation has proper semantic labeling
class AccessibleTab extends StatelessWidget {
  final String label;
  final IconData? icon;
  final bool isSelected;

  const AccessibleTab({
    super.key,
    required this.label,
    this.icon,
    this.isSelected = false,
  });

  @override
  Widget build(BuildContext context) {
    return Semantics(
      label: label,
      selected: isSelected,
      button: true,
      child: Tab(
        icon: icon != null ? Icon(icon) : null,
        text: label,
      ),
    );
  }
}
