import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-marketplace-escrow-bank',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './marketplace-escrow-bank.component.html',
  styleUrl: './marketplace-escrow-bank.component.css',
  host: {
    class: 'block w-full'
  }
})
export class MarketplaceEscrowBankComponent {}
