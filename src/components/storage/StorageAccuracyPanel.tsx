import { Database, HardDrive, ShieldCheck } from "lucide-react";
import { Card } from "../ui/Card";

export function StorageAccuracyPanel() {
  return (
    <Card className="p-5">
      <div className="flex items-center gap-2 text-sm font-semibold"><ShieldCheck className="text-sage-300" size={17} />How DiskSage reports space</div>
      <div className="mt-4 grid gap-3 md:grid-cols-3">
        <Metric icon={Database} label="Logical size" text="The apparent length of files. Sparse or compressed files can use less physical space." />
        <Metric icon={HardDrive} label="On disk" text="Allocated filesystem blocks. This is the most useful number for comparing current usage." />
        <Metric icon={ShieldCheck} label="Estimated reclaimable" text="Only allocated bytes in a reviewed cleanup plan—not every byte shown by an analysis." />
      </div>
      <p className="mt-4 border-t border-line pt-4 text-xs leading-5 text-muted">Free space can change later than expected while items remain in Trash, APFS snapshots retain blocks, or a sparse virtual disk such as Docker.raw keeps its maximum capacity.</p>
    </Card>
  );
}

function Metric({ icon: Icon, label, text }: { icon: typeof Database; label: string; text: string }) {
  return <div className="rounded-xl border border-line bg-white/[0.02] p-4"><Icon className="text-sage-300" size={17} /><p className="mt-3 text-sm font-semibold">{label}</p><p className="mt-1 text-xs leading-5 text-muted">{text}</p></div>;
}
