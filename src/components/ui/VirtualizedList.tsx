import { useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

type VirtualizedListProps<T> = {
  items: T[];
  itemKey: (item: T) => string;
  estimateSize: (item: T) => number;
  renderItem: (item: T, index: number) => React.ReactNode;
  label: string;
  className?: string;
};

export function VirtualizedList<T>({ items, itemKey, estimateSize, renderItem, label, className = "" }: VirtualizedListProps<T>) {
  const parentRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => parentRef.current,
    estimateSize: (index) => estimateSize(items[index]),
    getItemKey: (index) => itemKey(items[index]),
    overscan: 5,
    useFlushSync: false,
  });

  if (items.length < 100) {
    return <div className={className} role="list" aria-label={label}>{items.map((item, index) => <div role="listitem" key={itemKey(item)}>{renderItem(item, index)}</div>)}</div>;
  }

  return (
    <div ref={parentRef} className={`h-[min(68vh,720px)] overflow-auto pr-2 ${className}`} role="list" aria-label={`${label}, virtualized ${items.length} items`} tabIndex={0}>
      <div className="relative w-full" style={{ height: `${virtualizer.getTotalSize()}px` }}>
        {virtualizer.getVirtualItems().map((row) => (
          <div key={row.key} data-index={row.index} ref={virtualizer.measureElement} className="absolute left-0 top-0 w-full pb-3" role="listitem" style={{ transform: `translateY(${row.start}px)` }}>
            {renderItem(items[row.index], row.index)}
          </div>
        ))}
      </div>
    </div>
  );
}
