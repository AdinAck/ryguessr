export default interface AlertProps {
  title: string;
  description: string;
  duration?: number;
  onDismiss?: () => void;
}
