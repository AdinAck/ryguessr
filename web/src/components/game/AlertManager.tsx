import { useEffect } from "react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Info } from "lucide-react";
import AlertProps from "@/types/alert";
import { motion } from "framer-motion";

export function AutoDismissAlert({
  title,
  description,
  duration = 5000,
  onDismiss,
}: AlertProps) {
  useEffect(() => {
    const timer = setTimeout(() => {
      if (onDismiss) onDismiss();
    }, duration);

    return () => clearTimeout(timer);
  }, [duration]);

  return (
    <motion.div
      initial={{ opacity: 0, y: -20, scale: 0.95 }}
      animate={{ opacity: 1, y: 0, scale: 1 }}
      exit={{ opacity: 0, y: -50 }}
      transition={{ duration: 0.2, ease: "easeOut" }}
      className="w-full pointer-events-auto"
    >
      <Alert className={`shadow-md`}>
        <Info />
        <AlertTitle>{title}</AlertTitle>
        <AlertDescription>{description}</AlertDescription>
      </Alert>
    </motion.div>
  );
}
