import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-marketplace-exchange-resources',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './marketplace-exchange-resources.component.html',
  styleUrl: './marketplace-exchange-resources.component.css',
  host: {
    class: 'block w-full'
  }
})
export class MarketplaceExchangeResourcesComponent {}
