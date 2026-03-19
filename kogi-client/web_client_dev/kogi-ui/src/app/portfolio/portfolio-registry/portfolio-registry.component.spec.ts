import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PortfolioRegistryComponent } from './portfolio-registry.component';

describe('PortfolioRegistryComponent', () => {
  let component: PortfolioRegistryComponent;
  let fixture: ComponentFixture<PortfolioRegistryComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PortfolioRegistryComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(PortfolioRegistryComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
