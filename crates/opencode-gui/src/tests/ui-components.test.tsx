import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Button } from '../components/ui/button';
import { Input } from '../components/ui/input';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card';
import { Progress } from '../components/ui/progress';
import { Label } from '../components/ui/label';

describe('UI Components', () => {
  describe('Button', () => {
    it('renders with default props', () => {
      render(<Button>Click me</Button>);
      expect(screen.getByRole('button')).toHaveTextContent('Click me');
    });

    it('renders with different variants', () => {
      const { rerender } = render(<Button variant="destructive">Delete</Button>);
      expect(screen.getByRole('button')).toHaveClass('bg-destructive');

      rerender(<Button variant="outline">Outline</Button>);
      expect(screen.getByRole('button')).toHaveClass('border');

      rerender(<Button variant="secondary">Secondary</Button>);
      expect(screen.getByRole('button')).toHaveClass('bg-secondary');
    });

    it('renders with different sizes', () => {
      const { rerender } = render(<Button size="sm">Small</Button>);
      expect(screen.getByRole('button')).toHaveClass('h-9');

      rerender(<Button size="lg">Large</Button>);
      expect(screen.getByRole('button')).toHaveClass('h-11');
    });

    it('can be disabled', () => {
      render(<Button disabled>Disabled</Button>);
      expect(screen.getByRole('button')).toBeDisabled();
    });
  });

  describe('Input', () => {
    it('renders with placeholder', () => {
      render(<Input placeholder="Enter text" />);
      expect(screen.getByPlaceholderText('Enter text')).toBeInTheDocument();
    });

    it('accepts different input types', () => {
      render(<Input type="password" data-testid="password-input" />);
      expect(screen.getByTestId('password-input')).toHaveAttribute('type', 'password');
    });

    it('can be disabled', () => {
      render(<Input disabled data-testid="disabled-input" />);
      expect(screen.getByTestId('disabled-input')).toBeDisabled();
    });
  });

  describe('Card', () => {
    it('renders card with header and content', () => {
      render(
        <Card>
          <CardHeader>
            <CardTitle>Test Card</CardTitle>
          </CardHeader>
          <CardContent>
            <p>Card content</p>
          </CardContent>
        </Card>
      );

      expect(screen.getByText('Test Card')).toBeInTheDocument();
      expect(screen.getByText('Card content')).toBeInTheDocument();
    });

    it('applies correct CSS classes', () => {
      render(
        <Card data-testid="card">
          <CardHeader data-testid="header">
            <CardTitle data-testid="title">Title</CardTitle>
          </CardHeader>
        </Card>
      );

      expect(screen.getByTestId('card')).toHaveClass('rounded-lg', 'border');
      expect(screen.getByTestId('header')).toHaveClass('flex', 'flex-col');
      expect(screen.getByTestId('title')).toHaveClass('text-2xl', 'font-semibold');
    });
  });

  describe('Progress', () => {
    it('renders with correct value', () => {
      render(<Progress value={50} data-testid="progress" />);
      const progressBar = screen.getByTestId('progress');
      expect(progressBar).toBeInTheDocument();
    });

    it('handles zero value', () => {
      render(<Progress value={0} data-testid="progress-zero" />);
      expect(screen.getByTestId('progress-zero')).toBeInTheDocument();
    });

    it('handles maximum value', () => {
      render(<Progress value={100} data-testid="progress-full" />);
      expect(screen.getByTestId('progress-full')).toBeInTheDocument();
    });

    it('renders without value prop', () => {
      render(<Progress data-testid="progress-undefined" />);
      expect(screen.getByTestId('progress-undefined')).toBeInTheDocument();
    });
  });

  describe('Label', () => {
    it('renders with text', () => {
      render(<Label>Test Label</Label>);
      expect(screen.getByText('Test Label')).toBeInTheDocument();
    });

    it('can be associated with form controls', () => {
      render(
        <div>
          <Label htmlFor="test-input">Label</Label>
          <Input id="test-input" />
        </div>
      );

      const label = screen.getByText('Label');
      const input = screen.getByRole('textbox');
      expect(label).toHaveAttribute('for', 'test-input');
      expect(input).toHaveAttribute('id', 'test-input');
    });
  });
});