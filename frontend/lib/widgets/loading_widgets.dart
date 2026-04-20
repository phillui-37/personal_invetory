import 'package:flutter/material.dart';

/// Simple shimmer/skeleton loading animation using built-in Flutter widgets
class ShimmerLoading extends StatefulWidget {
  final Widget child;
  final bool isLoading;
  final Duration animationDuration;

  const ShimmerLoading({
    super.key,
    required this.child,
    this.isLoading = true,
    this.animationDuration = const Duration(milliseconds: 1500),
  });

  @override
  State<ShimmerLoading> createState() => _ShimmerLoadingState();
}

class _ShimmerLoadingState extends State<ShimmerLoading>
    with SingleTickerProviderStateMixin {
  late AnimationController _animationController;

  @override
  void initState() {
    super.initState();
    _animationController = AnimationController(
      duration: widget.animationDuration,
      vsync: this,
    );

    if (widget.isLoading) {
      _animationController.repeat();
    }
  }

  @override
  void didUpdateWidget(ShimmerLoading oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.isLoading && !_animationController.isAnimating) {
      _animationController.repeat();
    } else if (!widget.isLoading && _animationController.isAnimating) {
      _animationController.stop();
    }
  }

  @override
  void dispose() {
    _animationController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (!widget.isLoading) {
      return widget.child;
    }

    return _ShimmerOverlay(
      animation: _animationController,
      child: widget.child,
    );
  }
}

class _ShimmerOverlay extends AnimatedWidget {
  final Widget child;

  const _ShimmerOverlay({
    required Animation<double> animation,
    required this.child,
  }) : super(listenable: animation);

  @override
  Widget build(BuildContext context) {
    final animation = listenable as Animation<double>;

    return Stack(
      children: [
        child,
        Positioned.fill(
          child: Transform.translate(
            offset: Offset(animation.value * 400 - 200, 0),
            child: Container(
              decoration: BoxDecoration(
                gradient: LinearGradient(
                  begin: Alignment.centerLeft,
                  end: Alignment.centerRight,
                  colors: [
                    Colors.transparent,
                    Colors.white.withOpacity(0.3),
                    Colors.transparent,
                  ],
                  stops: const [0, 0.5, 1],
                ),
              ),
            ),
          ),
        ),
      ],
    );
  }
}

/// Skeleton loader for list items
class SkeletonListItem extends StatelessWidget {
  final bool isLoading;

  const SkeletonListItem({
    super.key,
    this.isLoading = true,
  });

  @override
  Widget build(BuildContext context) {
    return ShimmerLoading(
      isLoading: isLoading,
      child: ListTile(
        leading: Container(
          width: 48,
          height: 48,
          decoration: BoxDecoration(
            color: Colors.grey[300],
            borderRadius: BorderRadius.circular(4),
          ),
        ),
        title: Container(
          width: double.infinity,
          height: 16,
          color: Colors.grey[300],
        ),
        subtitle: Padding(
          padding: const EdgeInsets.only(top: 8),
          child: Container(
            width: double.infinity,
            height: 12,
            color: Colors.grey[300],
          ),
        ),
      ),
    );
  }
}

/// Skeleton loader for cards
class SkeletonCard extends StatelessWidget {
  final double height;
  final bool isLoading;

  const SkeletonCard({
    super.key,
    this.height = 200,
    this.isLoading = true,
  });

  @override
  Widget build(BuildContext context) {
    return ShimmerLoading(
      isLoading: isLoading,
      child: Card(
        child: Container(
          height: height,
          width: double.infinity,
          decoration: BoxDecoration(
            color: Colors.grey[300],
            borderRadius: BorderRadius.circular(12),
          ),
        ),
      ),
    );
  }
}

/// Skeleton loader for text lines
class SkeletonText extends StatelessWidget {
  final int lines;
  final bool isLoading;
  final double lineHeight;

  const SkeletonText({
    super.key,
    this.lines = 3,
    this.isLoading = true,
    this.lineHeight = 14,
  });

  @override
  Widget build(BuildContext context) {
    return ShimmerLoading(
      isLoading: isLoading,
      child: Column(
        children: List.generate(
          lines,
          (index) => Padding(
            padding: EdgeInsets.only(bottom: index < lines - 1 ? 8 : 0),
            child: Container(
              width: index == lines - 1 ? 200 : double.infinity,
              height: lineHeight,
              color: Colors.grey[300],
            ),
          ),
        ),
      ),
    );
  }
}

/// Progress indicator with label for long operations
class OperationProgress extends StatelessWidget {
  final int completed;
  final int total;
  final String operationName;
  final String? currentItem;

  const OperationProgress({
    super.key,
    required this.completed,
    required this.total,
    required this.operationName,
    this.currentItem,
  });

  @override
  Widget build(BuildContext context) {
    final progress = total > 0 ? completed / total : 0.0;

    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          operationName,
          style: Theme.of(context).textTheme.titleMedium,
        ),
        if (currentItem != null) ...[
          SizedBox(height: 8),
          Text(
            currentItem!,
            style: Theme.of(context).textTheme.bodySmall,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
          ),
        ],
        SizedBox(height: 12),
        Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            LinearProgressIndicator(
              value: progress,
              minHeight: 8,
            ),
            SizedBox(height: 8),
            Text(
              '$completed / $total items',
              style: Theme.of(context).textTheme.labelSmall,
            ),
          ],
        ),
      ],
    );
  }
}

/// Full page skeleton loader
class SkeletonLoadingPage extends StatelessWidget {
  final bool isLoading;
  final int itemCount;

  const SkeletonLoadingPage({
    super.key,
    this.isLoading = true,
    this.itemCount = 5,
  });

  @override
  Widget build(BuildContext context) {
    if (!isLoading) {
      return SizedBox.shrink();
    }

    return ListView.builder(
      itemCount: itemCount,
      itemBuilder: (context, index) => Padding(
        padding: const EdgeInsets.all(8),
        child: SkeletonListItem(isLoading: true),
      ),
    );
  }
}

/// Loading state overlay dialog
class LoadingDialog extends StatelessWidget {
  final String message;
  final String? subMessage;
  final bool isDismissible;

  const LoadingDialog({
    super.key,
    required this.message,
    this.subMessage,
    this.isDismissible = false,
  });

  @override
  Widget build(BuildContext context) {
    return WillPopScope(
      onWillPop: () async => isDismissible,
      child: AlertDialog(
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            CircularProgressIndicator(),
            SizedBox(height: 16),
            Text(
              message,
              textAlign: TextAlign.center,
              style: Theme.of(context).textTheme.bodyMedium,
            ),
            if (subMessage != null) ...[
              SizedBox(height: 8),
              Text(
                subMessage!,
                textAlign: TextAlign.center,
                style: Theme.of(context).textTheme.labelSmall,
              ),
            ],
          ],
        ),
      ),
    );
  }
}

/// Batch operation progress display
class BatchOperationProgress extends StatelessWidget {
  final String operationType; // 'import', 'update', 'copy'
  final int itemsProcessed;
  final int totalItems;
  final List<String> currentItemLabels;
  final bool hasErrors;
  final int? estimatedSecondsRemaining;

  const BatchOperationProgress({
    super.key,
    required this.operationType,
    required this.itemsProcessed,
    required this.totalItems,
    this.currentItemLabels = const [],
    this.hasErrors = false,
    this.estimatedSecondsRemaining,
  });

  @override
  Widget build(BuildContext context) {
    final progress = totalItems > 0 ? itemsProcessed / totalItems : 0.0;
    final percentage = (progress * 100).toStringAsFixed(0);

    return Container(
      padding: EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Theme.of(context).cardColor,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(
          color: Colors.grey.shade300,
          width: 1,
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                'Batch ${operationType.toUpperCase()}',
                style: Theme.of(context).textTheme.titleMedium?.copyWith(
                  fontWeight: FontWeight.bold,
                ),
              ),
              Container(
                padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                decoration: BoxDecoration(
                  color: hasErrors ? Colors.orange : Colors.green,
                  borderRadius: BorderRadius.circular(4),
                ),
                child: Text(
                  '$percentage%',
                  style: TextStyle(
                    color: Colors.white,
                    fontWeight: FontWeight.bold,
                    fontSize: 12,
                  ),
                ),
              ),
            ],
          ),
          SizedBox(height: 12),
          LinearProgressIndicator(
            value: progress,
            minHeight: 8,
          ),
          SizedBox(height: 12),
          Text(
            '$itemsProcessed / $totalItems items',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          if (currentItemLabels.isNotEmpty) ...[
            SizedBox(height: 8),
            ...currentItemLabels.take(2).map((label) => Padding(
              padding: EdgeInsets.only(top: 4),
              child: Text(
                '• $label',
                style: Theme.of(context).textTheme.labelSmall,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
            )),
            if (currentItemLabels.length > 2)
              Text(
                '• +${currentItemLabels.length - 2} more',
                style: Theme.of(context).textTheme.labelSmall,
              ),
          ],
          if (estimatedSecondsRemaining != null && estimatedSecondsRemaining! > 0) ...[
            SizedBox(height: 8),
            Text(
              'Est. time: ${estimatedSecondsRemaining}s',
              style: Theme.of(context).textTheme.labelSmall,
            ),
          ],
        ],
      ),
    );
  }
}
